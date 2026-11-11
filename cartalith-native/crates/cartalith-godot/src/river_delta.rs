//! **River deltas on the painted path** (`RIVERS_IN_MAP_SCOPE.md` RIM-4 and
//! owner question 3, Ruling BU; redesigned 2026-11-11 on the owner's *"deltas
//! don't look natural"*, `DELTA_GEOMETRY_RESEARCH.md`).
//!
//! A real river that meets the sea or a lake with a heavy load splits into a
//! tree of distributaries; a drawn centreline that simply stops at the shore
//! reads as a pipe. [`delta_fans`] derives, from the drawn network and the
//! water the map draws, a set of extra runs that the painted path adds to its
//! distance field (`river_field::build_with`) and its colour field
//! (`river_stroke::rasterize_colour_field_with`).
//!
//! **The shape** (the redesign; `DELTA_GEOMETRY_RESEARCH.md` section 3 has
//! the reasoning and the sources). The river's own drawn course from the apex
//! to the sea is the delta's main channel and is never redrawn. Distributaries
//! leave it at points along that course ([`DeltaKind::offtakes`] of them, on
//! alternating sides from a seeded first side), and each distributary may
//! split again at its end ([`DeltaKind::generations`] deep): a mouth-bar
//! bifurcation, every [`BIFURCATION_LENGTH_WIDTHS`] channel widths, into two
//! daughters that share the parent's discharge UNEQUALLY (the minor one takes
//! [`MINOR_SHARE`] of it) and part at about [`BIFURCATION_ANGLE_DEG`], the
//! larger daughter keeping closer to the parent's heading. Every channel is a
//! sine-generated curve ([`DeltaKind::meander_rad`]), not a straight line.
//! Widths follow hydraulic geometry: `w ~ sqrt(Q)`, so at every split the
//! daughters' squared widths sum to the parent's. A channel that would touch
//! another channel ([`clear_of_others`]) or meet no water within
//! [`MAX_REACH_OVER_SETBACK`] setbacks is dropped, and its sibling takes the
//! whole discharge -- except one that meets the river's own course after a
//! real run, which ends there as a loop (below). The plan shape follows the mouth's exposure
//! ([`exposure`], [`DeltaKind::of_exposure`]): a sheltered mouth (a bay, a
//! lake) grows the many-branched river-dominated tree, an open coast the
//! few-channel wave-dominated one (Galloway 1975's classes; the tidal class is
//! not modelled, since the engine has no tides).
//!
//! **RIM-7: the same fans on the deep-zoom tiles and in the export** (owner,
//! 2026-10-04). One gate, [`fans_drawn`], decides for the screen, the tiles
//! and the export whether a look has fans at all, and all three are handed
//! the same [`RiverGeometry`] (`river_field::PaintSource`), so the fan is the
//! screen's own at every zoom.
//!
//! **Which mouths** (unchanged by the redesign; `OUTSTANDING_WORK.md`'s Part A,
//! measured with `_riverzoom_probe.gd --rim4-mouths`). A river is a delta mouth
//! when the LAST piece of its run ends meeting water (`DrawnRun::reach`'s tail
//! -- never an inlet part-way along a run that goes on through a lake), its
//! `own_order` is at least [`MIN_ORDER`] and its discharge over that piece is
//! at least [`MIN_DISCHARGE_FRAC`] of the grid's cells.
//!
//! **Zoom (Ruling EN, RZ-5).** Every distributary is ranked as a network river
//! of its OWN discharge would be ([`RankOfDischarge`]), never ahead of its
//! apex river: the delta's largest channels appear with the river and its
//! smaller ones only as the zoom grows, by the one visibility law
//! (`river_stroke::river_visibility`) the field's B channel already carries --
//! no second texture. From [`DELTA_FULL_ZOOM`] (x16 over the opening view) a
//! fan is whole wherever its river is drawn ([`delta_rank`]; section 12, the
//! owner's *"the deltas seem to have gone"*).
//!
//! **Loops (section 12).** A distributary that meets the river's own course
//! after at least [`LOOP_MIN_CELLS`] ends there, rejoining it (an anabranch
//! round an island: Tejedor et al. 2015 count such loops on real deltas)
//! instead of being dropped by the no-contact rule; one that meets another
//! distributary is still dropped, so the no-comb rule stands.
//!
//! **What it is not.** Not generation: a pure function of the drawn geometry
//! and the drawn-water classification. Its "randomness" is a hash of each
//! mouth's apex position ([`Rng::for_mouth`]), so it is deterministic and the
//! same on every path; nothing a simulation reads comes from here and no
//! golden moves. Not the base view's vector stroke (`map_overlay.gd`, the
//! fallback for a look that keeps the stroke or a grid over the field's texel
//! budget): [`fans_drawn`] is false for such a look, so no path draws it a
//! fan. Not a change to any existing river: the fans are SEPARATE runs, so
//! with the switch off (`WorldGen::set_river_deltas`) the field, the colour
//! texture, every tile and every export are byte-identical to the same render
//! without this module, and the reference look (`js_reference`) draws none.
//!
//! Must never put a fan point on land farther than [`MAX_REACH_OVER_SETBACK`]
//! times its setback from the apex, never keep a channel that meets no water
//! within that reach (it would run along the shore instead of into it), never
//! let two channels touch away from their shared split, and never read a
//! missing discharge as zero.

use std::collections::HashMap;

use crate::river_stroke::{DrawnRun, RiverGeometry, EDGE_FRINGE_PX, FIELD_VIEW_PX, MIN_STROKE_PX};

/// The least own Strahler order of a river that gets a fan. **Labelled
/// judgement, from the data.** Part A (`_riverzoom_probe.gd --rim4-mouths`) on
/// three worlds -- the owner's 2048x1311 (seed 246371, density 1.55, geology
/// model), 1024x656 (483920) and 1536x983 (912345) -- found 2203, 171 and 69
/// river mouths, of which 138 (6.3 %), 10 (5.8 %) and 9 (13.0 %) are own order
/// >= 3, while 72.6 %, 64.9 % and 65.2 % of mouths are order 1 (headwater
/// trickles that happen to reach the coast) and 21.1 %, 29.2 % and 21.7 % order
/// 2. Order is the one measure that means the same on every world: discharge
/// as a share of the grid's cells has a median of 1.5e-5 on the owner world and
/// 4.4e-4 on the 1024x656 one (30x), because it counts upstream cells.
/// Strahler order 3 is "has absorbed two order-2 rivers": the first rank that
/// is a river system's trunk rather than one of its streams. **Known cost of
/// choosing order**: the biggest-discharge mouths are not always high order
/// (the owner world's largest, 40 214 cells = 1.5 % of its grid, is own order
/// 1, a single-run outlet), so some large rivers get no fan (open, not settled
/// here).
pub const MIN_ORDER: i16 = 3;

/// The least discharge, as a fraction of the grid's cell count, a mouth needs
/// to get a fan. **Labelled judgement, from the data** (`_riverzoom_probe.gd
/// --rim4-mouths`): order alone admits topological stubs -- the order >= 3
/// mouths of the three Part A worlds read down to 6.5, 147 and 2989 cells. 2e-4
/// is the owner world's own 90th percentile of the discharge of ALL its 2203
/// mouths (2.00e-4, 537 cells): a fan only where the river is in the top tenth
/// of the mouths of the world it is on. It keeps 62 of the owner world's 138
/// order >= 3 mouths by the engine's reading, all 10 of the 1024x656 world's and
/// all 9 of the 1536x983 world's. It is a share of the grid because discharge
/// is an upstream CELL count (weighted by rainfall).
pub const MIN_DISCHARGE_FRAC: f32 = 2.0e-4;

/// How far upstream of the shore crossing the delta's apex lies, in cells, per
/// Strahler order above 1: `(own_order - 1) * SETBACK_PER_ORDER_CELLS`, i.e.
/// 16 cells for order 3, 24 for order 4, 32 for order 5. **Labelled
/// judgement**: a delta's length scales with the river that built it, and a
/// large delta is some 200-400 km across (the Nile's ~240 km coast, the
/// Mississippi's bird's foot ~300 km); at the owner world's 19.6 km a cell that
/// is 12-20 cells. Tuned against the probe's frames, not measured; unchanged
/// by the 2026-11-11 redesign.
pub const SETBACK_PER_ORDER_CELLS: f32 = 8.0;

/// The shortest setback worth a fan, in cells. A river whose last piece is
/// shorter than this above its mouth (a stub beside a coast) gets no fan: four
/// cells is what leaves the branches room to part. **Labelled judgement.**
pub const MIN_SETBACK_CELLS: f32 = 4.0;

/// The most a channel may run, measured along its path from the apex, before
/// it must have met water, as a multiple of its fan's setback. A distributary
/// leaving the main channel at about 45 degrees from a point part-way down it
/// takes well under 1.4 setbacks to a straight shore; 1.6 leaves room for a
/// shore that bends and for the channel's own meander. A channel that has not
/// met water by then is DROPPED -- never drawn to the limit -- so a fan cannot
/// run along a coast. This is also the bound on how far onto land any fan
/// point lies from its apex (a path at most this long cannot end farther).
/// **Labelled judgement**, unchanged by the redesign.
pub const MAX_REACH_OVER_SETBACK: f32 = 1.6;

/// The full angle between the two daughters of a mouth-bar bifurcation, in
/// degrees. **Cited**: Coffey & Shaw 2017 (GRL 44, 11 427-11 436, "Congruent
/// bifurcation angles in river delta and tributary channel networks") predict
/// about 72 degrees and report 70.4 on average over 222 field bifurcations in
/// ten deltas (68.3 +- 8.7 in experiments). SECONDARY: the figures were read in
/// abstracts and a citing paper, not the paper itself
/// (`DELTA_GEOMETRY_RESEARCH.md` section 1.3). The old fan's fixed 22 and 45
/// degree pairs about one straight heading were what read as a comb.
pub const BIFURCATION_ANGLE_DEG: f32 = 70.0;

/// The spread drawn about [`BIFURCATION_ANGLE_DEG`] at each split, in
/// degrees, uniform in `+-` this. **Labelled judgement** sized to the cited
/// experimental scatter (+- 8.7 degrees, one standard deviation; a uniform of
/// half-width 12 has a standard deviation of 6.9), so no two splits share an
/// angle and the fan has no regular spacing.
pub const ANGLE_JITTER_DEG: f32 = 12.0;

/// The minor daughter's share of its parent's discharge at a split (or of the
/// main channel's at an offtake), drawn uniformly in this range. **Labelled
/// judgement, qualitatively cited**: Edmonds & Slingerland 2008 (WRR 44,
/// W09426, "Stability of delta distributary networks and their bifurcations")
/// find only the ASYMMETRIC bifurcation equilibria stable, so equal splits are
/// the exception; no numeric ratio was found (`DELTA_GEOMETRY_RESEARCH.md`
/// section 1.4), so the range is chosen to be clearly unequal (never more than
/// 45 : 55) without starving the minor branch below what draws.
pub const MINOR_SHARE: (f32, f32) = (0.2, 0.45);

/// The length of a distributary from its start to the mouth bar where it
/// splits, in its own widths. **Cited, secondary**: Jerolmack & Swenson 2007
/// (GRL 34, L23402) restate Edmonds & Slingerland 2007's mouth-bar scaling as
/// `L = a B`, `B` the channel's width and `a` "of order ten". Because width
/// goes as the root of discharge, a daughter's segment is shorter than its
/// parent's by the root of its share, which is the "widths, depths and lengths
/// decrease with successive bifurcations" Edmonds & Slingerland report.
pub const BIFURCATION_LENGTH_WIDTHS: f32 = 10.0;

/// The shortest segment a channel runs before it may split, in cells.
/// **Labelled judgement**: drawn widths here are a fraction of a cell, so ten
/// widths can be under one cell; three cells is what lets two daughters parting
/// at [`BIFURCATION_ANGLE_DEG`] stand clear of each other before the next split
/// ([`CLEAR_CELLS`]).
pub const MIN_SEGMENT_CELLS: f32 = 3.0;

/// A meander's wavelength in its channel's widths. **Cited, unconfirmed
/// here**: Leopold & Wolman 1960 (GSA Bulletin 71, "River meanders") give
/// meander wavelengths of about 7 to 11 channel widths; 11 is taken because
/// distributaries are long, gentle bends (research section 4.2).
pub const MEANDER_WAVELENGTH_WIDTHS: f32 = 11.0;

/// The shortest meander wavelength drawn, in cells. **Labelled judgement**: a
/// wavelength under four cells (two vertices a half-wave at
/// [`VERTEX_EVERY_STEPS`]) would alias into zigzags at the zooms the fan first
/// shows at.
pub const MIN_WAVELENGTH_CELLS: f32 = 4.0;

/// How much a terminal distributary (one that splits no further) narrows from
/// its start to its tip, as a fraction of its start width. **Labelled
/// judgement**: a terminal channel loses discharge to overbank flow and
/// crevasses; a channel that splits keeps its width so the split conserves
/// `w^2` exactly. The old fan's 0.4 applied to every branch.
pub const TERMINAL_TAPER: f32 = 0.25;

/// How close, in cells, a channel may come to any other channel (the main
/// channel included) once it is [`EXEMPT_CELLS`] from its own start: the
/// channel's point must have no other channel's point in the 3 x 3 cells round
/// it. **Labelled judgement**: one cell apart is what keeps two channels from
/// drawing as one smear or as two parallel strokes side by side, which is the
/// comb.
pub const CLEAR_CELLS: i32 = 1;

/// The least path a distributary must have run, cells, before meeting the
/// river's own course ends it as a loop rather than dropping it, and the least
/// straight distance from its start to where it rejoins. **Labelled
/// judgement** (section 12): twice [`EXEMPT_CELLS`], so the island a loop
/// encloses is at least six cells across -- about 44 pixels at x16 on the
/// owner world (7.4 pixels a cell), a shape that reads as an island and not a
/// thickened stroke.
pub const LOOP_MIN_CELLS: f32 = 6.0;

/// How far from its own start a channel ignores [`CLEAR_CELLS`], in cells.
/// **Labelled judgement**: two daughters parting at 58 degrees (the least
/// [`BIFURCATION_ANGLE_DEG`] minus [`ANGLE_JITTER_DEG`]) are 3 x 2 sin 29 = 2.9
/// cells apart after three cells, enough to clear the 3 x 3 neighbourhood.
pub const EXEMPT_CELLS: f32 = 3.0;

/// The march's step along a channel, in cells. **Labelled judgement**: half a
/// cell finds where a channel first stands on a water cell to better than the
/// cell the classification itself is made at.
pub const STEP_CELLS: f32 = 0.5;

/// How fast a channel's mean heading turns toward the way down to the water
/// ([`SeaField`]), in radians per cell of march. **Labelled judgement**: water
/// on a delta plain runs seaward, so a distributary that leaves its parent at
/// the bifurcation angle bends toward the shore over the next several cells
/// (0.06 rad a cell is 34 degrees over ten cells) instead of running on along
/// the coast; slow enough that the split's angle still shows, and not applied
/// over a channel's first [`EXEMPT_CELLS`]. Without it the
/// daughters' turns add up generation after generation until a channel heads
/// back inland.
pub const SEAWARD_TURN_RAD_PER_CELL: f32 = 0.06;

/// One vertex every this many steps (one cell) on land. A meander of
/// [`MIN_WAVELENGTH_CELLS`] keeps four vertices a wave.
const VERTEX_EVERY_STEPS: usize = 2;

/// Where along the main channel, from the apex (0) to the shore crossing (1),
/// distributaries may leave it. **Labelled judgement**: the first offtake no
/// nearer the apex than a seventh of the setback (the delta's head is one
/// channel), the last no nearer the shore than 0.75, so a distributary has room
/// to bend away before it meets the water.
pub const OFFTAKE_SPAN: (f32, f32) = (0.15, 0.75);

/// Exposure (the water share of the half-disc in front of a mouth,
/// [`exposure`]) below which a mouth is sheltered and grows the
/// river-dominated tree, and above which it is open and grows the
/// wave-dominated one; between the two, a lobate delta. **Labelled judgement,
/// a proxy**: Nienhuis et al. 2020 (Nature 577, 514-518) classify a delta by
/// `R = Q_river / Q_wave` (CONFIRMED in the paper's text), with wave-dominated
/// deltas where the wave flux exceeds the river's. The engine has no wave
/// field, and the open water in front of a mouth -- its fetch -- is what sets
/// how much wave energy can reach it: a lake, a bay or a strait is sheltered,
/// a straight open coast is not. The thresholds were chosen so the half-disc
/// of a straight coast (0.5 to 1.0 of it water depending on the angle the
/// river meets it at) reads open and a mouth into a narrow inlet reads
/// sheltered.
pub const EXPOSURE_SHELTERED: f32 = 0.55;
/// See [`EXPOSURE_SHELTERED`].
pub const EXPOSURE_OPEN: f32 = 0.8;

/// A delta's plan type, from its mouth's [`exposure`]. Galloway 1975's three
/// end members are river-, wave- and tide-dominated; the tidal one needs a tide
/// the engine does not have and is not drawn.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeltaKind {
    /// Sheltered water (a lake, a bay): many distributaries in a deep tree,
    /// low sinuosity -- the Mississippi's bird's foot and the Danube's lobes.
    RiverDominated,
    /// Between the two: a lobate fan of a few branches.
    Lobate,
    /// An open coast: waves suppress mouth bars (Jerolmack & Swenson 2007,
    /// secondary) and leave about one channel beside the main one -- the
    /// cuspate Ebro, Tiber or Sao Francisco.
    WaveDominated,
}

impl DeltaKind {
    /// The type of a mouth of exposure `e` (see [`EXPOSURE_SHELTERED`]).
    pub fn of_exposure(e: f32) -> DeltaKind {
        if e < EXPOSURE_SHELTERED {
            DeltaKind::RiverDominated
        } else if e < EXPOSURE_OPEN {
            DeltaKind::Lobate
        } else {
            DeltaKind::WaveDominated
        }
    }

    /// Distributaries that leave the main channel, before any further split.
    /// One more from own order 5 (a larger river builds a larger delta).
    /// **Labelled judgement** on the classes' qualitative counts (research
    /// section 2: "about 1" on a wave-dominated delta, many on a
    /// river-dominated one; no per-type count was found in a source).
    pub fn offtakes(self, own_order: i16) -> usize {
        let base = match self {
            DeltaKind::RiverDominated => 2,
            DeltaKind::Lobate => 2,
            DeltaKind::WaveDominated => 1,
        };
        base + usize::from(own_order >= 5)
    }

    /// How many levels of distributary a delta of this type grows below the
    /// main channel: 1 is a distributary that never splits. **Labelled
    /// judgement**, as for [`Self::offtakes`]: a river-dominated tree of up to
    /// eight terminal channels per offtake, a lobate one of up to four, a
    /// wave-dominated one unbranched.
    pub fn generations(self) -> u8 {
        match self {
            DeltaKind::RiverDominated => 3,
            DeltaKind::Lobate => 2,
            DeltaKind::WaveDominated => 1,
        }
    }

    /// The amplitude of the channel's heading about its mean, in radians, of a
    /// sine-generated curve (Langbein & Leopold 1966). Its sinuosity is
    /// `1 / J0(amplitude)`: 1.06 at 0.45 and 1.03 at 0.35. **Labelled
    /// judgement within a cited band**: delta distributaries are low-sinuosity
    /// channels, under the 1.2 that research section 4.1's sources call high.
    pub fn meander_rad(self) -> f32 {
        match self {
            DeltaKind::RiverDominated => 0.45,
            DeltaKind::Lobate => 0.45,
            DeltaKind::WaveDominated => 0.35,
        }
    }
}

/// What one [`delta_fans`] call did, for `WorldGen::river_delta_stats` and the
/// probe. Every count is of MOUTHS (runs) unless it says channels or splits.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DeltaStats {
    /// Runs whose last piece ends meeting water.
    pub mouths: usize,
    /// Of those, own order >= [`MIN_ORDER`] with a discharge reading at or
    /// above [`MIN_DISCHARGE_FRAC`] of the grid.
    pub eligible: usize,
    /// Eligible mouths that carried no finite discharge reading at all (left
    /// out, never read as zero).
    pub no_discharge: usize,
    /// Eligible mouths given a fan (at least one channel kept).
    pub fans: usize,
    /// Channels kept, over all fans (one run each).
    pub branches: usize,
    /// Channels planned and dropped for meeting no water within
    /// [`MAX_REACH_OVER_SETBACK`] setbacks or leaving the map.
    pub dropped_dry: usize,
    /// Channels planned and dropped for touching another ([`CLEAR_CELLS`]).
    pub collided: usize,
    /// Bifurcations that kept both daughters.
    pub splits: usize,
    /// Channels kept as loops: they met the river's own course after
    /// [`LOOP_MIN_CELLS`] and end on it (section 12) instead of being counted
    /// in `collided`.
    pub rejoined: usize,
    /// Fans of each [`DeltaKind`]: river-dominated, lobate, wave-dominated.
    pub kinds: [usize; 3],
    /// Eligible mouths skipped because the river is under
    /// [`MIN_SETBACK_CELLS`] long above its shore crossing, has no dry point,
    /// or its apex stands on water.
    pub skipped: usize,
    /// Per fan, in output order: how many channels (runs) it put into the
    /// returned geometry, so a reader can split the runs back into mouths
    /// ([`fan_seen`]). Sums to `branches`.
    pub fan_sizes: Vec<u32>,
    /// Per fan, in output order: the apex river's own visibility rank (NaN
    /// when the network is unranked -- never a guessed 0).
    pub fan_apex_rank: Vec<f32>,
}

/// Whether grid cell `(i, j)` is drawn water -- the closure `river_draws`'
/// `cell_wet` is (out of the grid is not water).
pub type Wet<'a> = &'a dyn Fn(i64, i64) -> bool;

/// Whether the grid cell holding the point `p` is drawn water. Must never be
/// asked about a point outside the grid for anything but "not water": the
/// [`Wet`] closure owns that rule.
fn wet_at(wet: Wet, p: (f32, f32)) -> bool {
    wet(p.0.floor() as i64, p.1.floor() as i64)
}

/// The distance between two points, in cells.
fn dist(a: (f32, f32), b: (f32, f32)) -> f32 {
    ((a.0 - b.0).powi(2) + (a.1 - b.1).powi(2)).sqrt()
}

/// **A mouth's exposure**: the share of the cells in the half-disc of radius
/// `r` in front of the shore crossing `d` (facing `dir`, a unit vector) that
/// are drawn water, in `0..=1`; cells off the grid count as land. A proxy for
/// the fetch that waves reach the mouth over ([`EXPOSURE_SHELTERED`]). `None`
/// when the half-disc holds no cell (a radius under one cell).
pub fn exposure(d: (f32, f32), dir: (f32, f32), r: f32, wet: Wet) -> Option<f32> {
    let ri = r.ceil() as i64;
    let (mut n, mut w) = (0u32, 0u32);
    for dy in -ri..=ri {
        for dx in -ri..=ri {
            let (fx, fy) = (dx as f32, dy as f32);
            if fx * fx + fy * fy > r * r || fx * dir.0 + fy * dir.1 <= 0.0 {
                continue;
            }
            n += 1;
            if wet((d.0 + fx).floor() as i64, (d.1 + fy).floor() as i64) {
                w += 1;
            }
        }
    }
    (n > 0).then(|| w as f32 / n as f32)
}

/// A small deterministic generator (SplitMix64, Steele, Lea & Flood 2014),
/// seeded per mouth so a fan depends only on where its river reaches the sea,
/// never on the order the network lists its runs in.
struct Rng(u64);

impl Rng {
    /// The generator for the mouth whose apex is `a` and whose river is of
    /// own order `order`: a hash of the coordinates' bits, so two runs of the
    /// same world draw the same fan and a moved apex draws a new one.
    fn for_mouth(a: (f32, f32), order: i16) -> Rng {
        Rng(((a.0.to_bits() as u64) << 32 | a.1.to_bits() as u64) ^ (order as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15))
    }
    /// The next value in `[0, 1)`.
    fn unit(&mut self) -> f32 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^= z >> 31;
        (z >> 40) as f32 / (1u64 << 24) as f32
    }
    /// A value uniform in `[lo, hi)`.
    fn range(&mut self, lo: f32, hi: f32) -> f32 {
        lo + (hi - lo) * self.unit()
    }
}

/// **The visibility rank a network river of a given discharge has** (Ruling
/// EN's `u`): the share of the drawn network's length whose discharge is at
/// least that, so a distributary carrying a fifth of its river's water appears
/// at the zoom a river that size appears at (RZ-5's "fans fade in by their
/// on-screen reach", through the existing law). `None` when the network
/// carries no ranks (a hand-built test network): every fan then takes its
/// apex's rank, i.e. is unranked.
///
/// **Why the length share and not the network's own ranks** (section 12,
/// 2026-11-11; the owner: *"the deltas seem to have gone"*). Until then the
/// table read the network's `(discharge, rank)` pairs with a running maximum
/// of the rank down the discharges. Ruling EQ item 3 (`hide_short_stubs`)
/// raises a short run's ranks toward 1 whatever its discharge, so one short
/// run of large discharge carried rank 1 to every smaller discharge: measured
/// on the owner world, the median fan channel ranked 1.0 against a median
/// apex of 0.21, and no channel was drawn below x64. The length share is the
/// definition `visibility_ranks` computes, read off discharge alone, so a rank
/// raised for a reason other than discharge cannot leak into it.
pub struct RankOfDischarge {
    /// `(discharge, share of drawn length at or above it)`, discharge
    /// descending, the share non-decreasing: less water never ranks ahead of
    /// more.
    pairs: Vec<(f32, f32)>,
}

impl RankOfDischarge {
    /// Builds the table from every drawn network point with a finite
    /// discharge, each weighted by the step from the point before it; empty
    /// (every lookup `None`) when no run carries ranks, i.e. the network is
    /// unranked.
    pub fn of(geom: &RiverGeometry) -> Self {
        if geom.runs.iter().all(|r| r.ranks.is_empty()) {
            return RankOfDischarge { pairs: Vec::new() };
        }
        let mut pairs: Vec<(f32, f32)> = Vec::new();
        for r in &geom.runs {
            for (k, &q) in r.discharge.iter().enumerate().take(r.pts.len()) {
                if k > 0 && q.is_finite() {
                    pairs.push((q, dist(r.pts[k], r.pts[k - 1])));
                }
            }
        }
        pairs.sort_by(|a, b| b.0.total_cmp(&a.0));
        let total: f32 = pairs.iter().map(|p| p.1).sum();
        if total <= 0.0 {
            return RankOfDischarge { pairs: Vec::new() };
        }
        let mut acc = 0.0f32;
        for p in &mut pairs {
            acc += p.1;
            p.1 = (acc / total).min(1.0);
        }
        RankOfDischarge { pairs }
    }
    /// The rank of discharge `q`: the share of drawn length carrying at least
    /// `q` (`0` above the largest discharge); `None` with an empty table.
    pub fn rank(&self, q: f32) -> Option<f32> {
        if self.pairs.is_empty() {
            return None;
        }
        let k = self.pairs.partition_point(|p| p.0 >= q);
        Some(if k == 0 { 0.0 } else { self.pairs[k - 1].1 })
    }
}

/// The zoom, over the opening fit view, by which every channel of a fan whose
/// river is drawn is drawn too. **The owner's, relayed 2026-11-11** (on the
/// x16 frames, *"the deltas seem to have gone"*; the coordinator's fix: "a
/// delta clearly reads at x16 and deeper"): a channel's rank is held at or
/// below the share the law reaches at this zoom ([`delta_rank`]), so between
/// the opening view and here the smaller channels fade in by their discharge
/// (Ruling EN's RZ-5) and from here on a fan is whole wherever its river is.
pub const DELTA_FULL_ZOOM: f32 = 16.0;

/// **A fan channel's visibility rank**: the rank of its own discharge
/// (`own`, [`RankOfDischarge`]), never ahead of its apex river's (`apex`,
/// RZ-2: a channel cannot show where its river does not), and never later than
/// the share the law reaches at [`DELTA_FULL_ZOOM`] for a map of start share
/// `start` (`river_stroke::start_share`) -- unless its river itself is later,
/// when it is the river's. `own` `None` (an unranked network) gives `apex`.
/// Independent of the window: the share at a zoom over the fit view depends on
/// the start share only.
pub fn delta_rank(own: Option<f32>, apex: f32, start: f32) -> f32 {
    let full = crate::river_stroke::visible_share_at(DELTA_FULL_ZOOM, 1.0, start);
    own.map_or(apex, |u| u.min(full).max(apex))
}

/// One channel of a fan being grown: its centreline, its share of the apex
/// river's discharge and its daughters.
struct Chan {
    /// Centreline, grid cells, from its start (on its parent) to its end.
    pts: Vec<(f32, f32)>,
    /// Share of the main channel's discharge at the apex, `0..1`.
    q: f32,
    /// The main channel's full drawn width at this channel's root offtake,
    /// cells: widths are `w_ref * sqrt(q)`.
    w_ref: f32,
    /// Indices of its kept daughters.
    kids: Vec<usize>,
    /// Kept (reached water, or split into a kept daughter).
    alive: bool,
    /// Ends on the river's own course (a loop), not on water.
    rejoins: bool,
}

/// **The distance to drawn water round one mouth**, on the grid's cells in a
/// square window: a two-pass chamfer distance transform (Borgefors 1986, the
/// 3-4 chamfer scaled to cells) from every wet cell, so its downhill direction
/// points the shortest way to the water -- seaward across a delta plain, and
/// round a cape or into a bay as the coast bends. Cells off the grid are land.
/// Never read outside its window ([`Self::downhill`] answers `None` there).
struct SeaField {
    /// The window's first cell.
    x0: i64,
    /// See `x0`.
    y0: i64,
    /// Its side, cells.
    n: usize,
    /// Distance to the nearest wet cell, cells; `f32::INFINITY` with no water
    /// in the window.
    d: Vec<f32>,
}

impl SeaField {
    /// The field over the window of half-side `r` cells round `c`.
    fn around(c: (f32, f32), r: f32, gw: usize, gh: usize, wet: Wet) -> SeaField {
        let r = r.ceil() as i64;
        let (x0, y0) = (c.0.floor() as i64 - r, c.1.floor() as i64 - r);
        let n = (2 * r + 1) as usize;
        let mut d = vec![f32::INFINITY; n * n];
        for j in 0..n {
            for i in 0..n {
                let (x, y) = (x0 + i as i64, y0 + j as i64);
                if x >= 0 && y >= 0 && (x as usize) < gw && (y as usize) < gh && wet(x, y) {
                    d[j * n + i] = 0.0;
                }
            }
        }
        // 1 and sqrt 2: the chamfer's axial and diagonal steps, in cells.
        let (a, b) = (1.0f32, std::f32::consts::SQRT_2);
        for j in 0..n {
            for i in 0..n {
                let mut v = d[j * n + i];
                if i > 0 { v = v.min(d[j * n + i - 1] + a); }
                if j > 0 {
                    v = v.min(d[(j - 1) * n + i] + a);
                    if i > 0 { v = v.min(d[(j - 1) * n + i - 1] + b); }
                    if i + 1 < n { v = v.min(d[(j - 1) * n + i + 1] + b); }
                }
                d[j * n + i] = v;
            }
        }
        for j in (0..n).rev() {
            for i in (0..n).rev() {
                let mut v = d[j * n + i];
                if i + 1 < n { v = v.min(d[j * n + i + 1] + a); }
                if j + 1 < n {
                    v = v.min(d[(j + 1) * n + i] + a);
                    if i + 1 < n { v = v.min(d[(j + 1) * n + i + 1] + b); }
                    if i > 0 { v = v.min(d[(j + 1) * n + i - 1] + b); }
                }
                d[j * n + i] = v;
            }
        }
        SeaField { x0, y0, n, d }
    }

    /// The field at cell `(i, j)` of the window, `None` outside it or where
    /// no water is in reach.
    fn at(&self, i: i64, j: i64) -> Option<f32> {
        if i < 0 || j < 0 || i as usize >= self.n || j as usize >= self.n {
            return None;
        }
        let v = self.d[j as usize * self.n + i as usize];
        v.is_finite().then_some(v)
    }

    /// The heading, radians, of steepest descent toward the water at `p`
    /// (central differences over the cells either side), or `None` outside
    /// the window, with no water in reach, or on a flat (no gradient).
    fn downhill(&self, p: (f32, f32)) -> Option<f32> {
        let (i, j) = (p.0.floor() as i64 - self.x0, p.1.floor() as i64 - self.y0);
        let gx = self.at(i + 1, j)? - self.at(i - 1, j)?;
        let gy = self.at(i, j + 1)? - self.at(i, j - 1)?;
        (gx != 0.0 || gy != 0.0).then(|| (-gy).atan2(-gx))
    }
}

/// Why a channel was not kept.
enum Fail {
    /// No water within the reach limit, or off the grid.
    Dry,
    /// Touched another channel.
    Collided,
}

/// The state of one mouth's fan while it grows.
struct Grow<'a> {
    wet: Wet<'a>,
    gw: usize,
    gh: usize,
    /// Path length from the apex past which a channel still on land fails.
    lmax: f32,
    kind: DeltaKind,
    rng: Rng,
    /// Grid cell -> the channel that first stood in it (`u32::MAX` the main
    /// channel). Land cells only.
    occ: HashMap<(i32, i32), u32>,
    /// The cells each channel added to `occ`, to take back if it fails.
    owned: Vec<Vec<(i32, i32)>>,
    chans: Vec<Chan>,
    /// Stop-width rule: `cap(half_width_cells)` = how far a channel carries
    /// on over water so its end is wholly in it.
    cap_scale: (f32, f32, f32),
    /// The way down to the water round this mouth.
    sea: SeaField,
    /// The river's own course from the apex to the shore (the main channel),
    /// sampled every [`STEP_CELLS`]: where a loop rejoins it.
    course: Vec<(f32, f32)>,
    /// Channels kept as loops onto the main channel.
    rejoined: usize,
    dropped_dry: usize,
    collided: usize,
    splits: usize,
}

/// Whether the cell of `p` or a cell round it ([`CLEAR_CELLS`]) is the main
/// channel's (`u32::MAX` in the occupancy grid).
fn touches_main(occ: &HashMap<(i32, i32), u32>, p: (f32, f32)) -> bool {
    let (ci, cj) = (p.0.floor() as i32, p.1.floor() as i32);
    (-CLEAR_CELLS..=CLEAR_CELLS).any(|dj| (-CLEAR_CELLS..=CLEAR_CELLS).any(|di| occ.get(&(ci + di, cj + dj)) == Some(&u32::MAX)))
}

/// Whether the cell of `p` and the cells round it ([`CLEAR_CELLS`]) hold no
/// channel but `own`.
fn clear_of_others(occ: &HashMap<(i32, i32), u32>, p: (f32, f32), own: u32) -> bool {
    let (ci, cj) = (p.0.floor() as i32, p.1.floor() as i32);
    for dj in -CLEAR_CELLS..=CLEAR_CELLS {
        for di in -CLEAR_CELLS..=CLEAR_CELLS {
            if let Some(&o) = occ.get(&(ci + di, cj + dj)) {
                if o != own {
                    return false;
                }
            }
        }
    }
    true
}

impl Grow<'_> {
    /// The water cap for a channel of full width `w` cells: its half-width
    /// times the preset's multiplier, floored at half the stroke's minimum
    /// drawn width, plus the antialiasing fringe, so the end is a drawn line
    /// wholly on water at the coarsest view.
    fn cap(&self, w: f32) -> f32 {
        let (rw, s_min, _) = self.cap_scale;
        (0.5 * w * rw).max(0.5 * MIN_STROKE_PX * s_min) + EDGE_FRINGE_PX * s_min
    }

    /// Adds channel `id`'s land points to the occupancy grid (each cell keeps
    /// its first owner).
    fn commit(&mut self, id: usize) {
        let pts = std::mem::take(&mut self.chans[id].pts);
        for &p in &pts {
            if !wet_at(self.wet, p) {
                let c = (p.0.floor() as i32, p.1.floor() as i32);
                if let std::collections::hash_map::Entry::Vacant(v) = self.occ.entry(c) {
                    v.insert(id as u32);
                    self.owned[id].push(c);
                }
            }
        }
        self.chans[id].pts = pts;
    }

    /// Takes channel `id`'s cells back out of the occupancy grid.
    fn uncommit(&mut self, id: usize) {
        for c in std::mem::take(&mut self.owned[id]) {
            self.occ.remove(&c);
        }
    }

    /// **Grows one channel** from `start` (on its parent) heading `h0`
    /// radians, carrying share `q` of the apex discharge, `path0` cells of path
    /// from the apex already behind it, at depth `depth` (1 = it leaves the main
    /// channel). It marches as a sine-generated curve of [`DeltaKind::meander_rad`]
    /// and wavelength [`MEANDER_WAVELENGTH_WIDTHS`] widths, the meander faded in
    /// over the first half-wave so it leaves its parent at exactly `h0`; after
    /// [`BIFURCATION_LENGTH_WIDTHS`] widths still on land it splits
    /// ([`Self::split`]) if its depth allows, and carries on unsplit if no
    /// daughter is kept. It stops [`Self::cap`] past the first water it
    /// reaches. Returns its index when kept; a channel that fails takes its
    /// cells back.
    #[allow(clippy::too_many_arguments)]
    fn grow(&mut self, start: (f32, f32), h0: f32, q: f32, w_ref: f32, path0: f32, depth: u8) -> Result<usize, Fail> {
        let id = self.chans.len();
        self.chans.push(Chan { pts: vec![start], q, w_ref, kids: Vec::new(), alive: false, rejoins: false });
        self.owned.push(Vec::new());
        let w = w_ref * q.sqrt();
        let lambda = (MEANDER_WAVELENGTH_WIDTHS * w).max(MIN_WAVELENGTH_CELLS);
        let amp = self.kind.meander_rad();
        let phase = self.rng.range(0.0, std::f32::consts::TAU);
        // The segment to the mouth bar, +-20 % so sibling splits do not line
        // up (labelled judgement).
        let seg = (BIFURCATION_LENGTH_WIDTHS * w).max(MIN_SEGMENT_CELLS) * self.rng.range(0.8, 1.2);
        let cap = self.cap(w);
        let mut can_split = depth < self.kind.generations();
        let (mut p, mut s, mut k) = (start, 0.0f32, 0usize);
        let mut on_water_since: Option<f32> = None;
        // The channel's mean heading: it starts at `h0` and turns toward the
        // water at most `SEAWARD_TURN_RAD_PER_CELL` a cell.
        let mut hm = h0;
        let fail = |g: &mut Self, f: Fail| {
            g.uncommit(id);
            Err(f)
        };
        loop {
            s += STEP_CELLS;
            k += 1;
            let ramp = (2.0 * s / lambda).min(1.0);
            // Not over the first EXEMPT_CELLS: the daughters leave the split at
            // its angle before they bend seaward (on the owner world, steering
            // from the first step read the split angle as 57 degrees).
            if s > EXEMPT_CELLS && let Some(down) = self.sea.downhill(p) {
                let dh = (down - hm + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU) - std::f32::consts::PI;
                let lim = SEAWARD_TURN_RAD_PER_CELL * STEP_CELLS;
                hm += dh.clamp(-lim, lim);
            }
            let h = hm + amp * ramp * (std::f32::consts::TAU * s / lambda + phase).sin();
            let next = (p.0 + h.cos() * STEP_CELLS, p.1 + h.sin() * STEP_CELLS);
            if next.0 < 0.0 || next.1 < 0.0 || next.0 >= self.gw as f32 || next.1 >= self.gh as f32 {
                return fail(self, Fail::Dry);
            }
            let wet_now = wet_at(self.wet, next);
            if let Some(s_w) = on_water_since {
                // Stay on water for the cap, then stop; leave it and stop at
                // the last wet point (a strait: the channel has crossed it).
                if !wet_now || s - s_w >= cap {
                    self.commit(id);
                    self.chans[id].alive = true;
                    return Ok(id);
                }
                p = next;
                self.chans[id].pts.push(p);
                continue;
            }
            if path0 + s > self.lmax {
                return fail(self, Fail::Dry);
            }
            if !wet_now && s > EXEMPT_CELLS && !clear_of_others(&self.occ, next, id as u32) {
                // Section 12: meeting the river's own course after a real run
                // ends the channel there, a loop round an island; anything else
                // (another distributary, or too soon) is still dropped.
                if s >= LOOP_MIN_CELLS && touches_main(&self.occ, next) {
                    if let Some(&m) = self.course.iter().min_by(|a, b| dist(**a, next).total_cmp(&dist(**b, next))) {
                        if dist(m, start) >= LOOP_MIN_CELLS {
                            self.chans[id].pts.push(m);
                            self.commit(id);
                            self.chans[id].alive = true;
                            self.chans[id].rejoins = true;
                            self.rejoined += 1;
                            return Ok(id);
                        }
                    }
                }
                return fail(self, Fail::Collided);
            }
            p = next;
            if wet_now {
                on_water_since = Some(s);
                self.chans[id].pts.push(p);
                if cap <= 0.0 {
                    self.commit(id);
                    self.chans[id].alive = true;
                    return Ok(id);
                }
                continue;
            }
            if k % VERTEX_EVERY_STEPS == 0 {
                self.chans[id].pts.push(p);
            }
            if can_split && s >= seg {
                if *self.chans[id].pts.last().unwrap() != p {
                    self.chans[id].pts.push(p);
                }
                self.commit(id);
                if self.split(id, p, h, q, w_ref, path0 + s, depth) {
                    self.chans[id].alive = true;
                    return Ok(id);
                }
                can_split = false;
            }
        }
    }

    /// **A mouth-bar bifurcation** at the end `p` of channel `id` (heading `h`):
    /// the minor daughter takes a share drawn from [`MINOR_SHARE`] of `q`, the
    /// full angle between the two is [`BIFURCATION_ANGLE_DEG`] `+-`
    /// [`ANGLE_JITTER_DEG`], and each turns from `h` by the angle times the
    /// OTHER's share, so the larger keeps closer to the parent's heading (the
    /// momentum of the larger flow). The minor one's side is drawn. A daughter
    /// that fails gives its share to the other. True when one is kept.
    #[allow(clippy::too_many_arguments)]
    fn split(&mut self, id: usize, p: (f32, f32), h: f32, q: f32, w_ref: f32, path: f32, depth: u8) -> bool {
        let s = self.rng.range(MINOR_SHARE.0, MINOR_SHARE.1);
        let full = (BIFURCATION_ANGLE_DEG + self.rng.range(-ANGLE_JITTER_DEG, ANGLE_JITTER_DEG)).to_radians();
        let side = if self.rng.unit() < 0.5 { 1.0 } else { -1.0 };
        let mut kids = Vec::new();
        for (share, turn) in [(1.0 - s, -side * full * s), (s, side * full * (1.0 - s))] {
            match self.grow(p, h + turn, q * share, w_ref, path, depth + 1) {
                Ok(c) => kids.push(c),
                Err(f) => self.count(f),
            }
        }
        match kids.len() {
            2 => self.splits += 1,
            1 => self.give_whole(kids[0], q),
            _ => {}
        }
        self.chans[id].kids = kids;
        !self.chans[id].kids.is_empty()
    }

    /// Sets channel `c` and its whole subtree to carry `q` in place of the
    /// share it was grown with (its sibling was dropped), keeping the
    /// subtree's own proportions.
    fn give_whole(&mut self, c: usize, q: f32) {
        let f = q / self.chans[c].q;
        let mut stack = vec![c];
        while let Some(i) = stack.pop() {
            self.chans[i].q *= f;
            stack.extend(self.chans[i].kids.iter().copied());
        }
    }

    /// Counts a dropped channel by its cause.
    fn count(&mut self, f: Fail) {
        match f {
            Fail::Dry => self.dropped_dry += 1,
            Fail::Collided => self.collided += 1,
        }
    }
}

/// **The delta fans of a drawn network**, as a [`RiverGeometry`] of their own
/// (one run per channel) and what was done ([`DeltaStats`]).
///
/// For each run whose last piece ends meeting water (`reach` tail), of own
/// order >= [`MIN_ORDER`] and discharge >= [`MIN_DISCHARGE_FRAC`] of
/// `gw * gh`:
///
/// 1. the shore crossing `D` is the last point of that piece that is dry;
/// 2. the apex `A` is the point `U = (own_order - 1) *
///    [`SETBACK_PER_ORDER_CELLS`]` cells of arc length upstream of `D` (less if
///    the piece is shorter; the mouth is skipped under [`MIN_SETBACK_CELLS`]);
/// 3. the mouth's [`exposure`] over a half-disc of radius `U` in front of `D`
///    (facing along the chord `A -> D`) picks its [`DeltaKind`];
/// 4. [`DeltaKind::offtakes`] distributaries leave the river's own course
///    between `A` and `D` (spaced over [`OFFTAKE_SPAN`] with a drawn jitter,
///    on alternating sides), each taking a [`MINOR_SHARE`] of the river's
///    remaining discharge and turning from the course by
///    [`BIFURCATION_ANGLE_DEG`] times the river's remaining share; each grows
///    and splits as [`Grow::grow`] says;
/// 5. widths: `w_ref * sqrt(q)` (`w_ref` the river's width at the offtake,
///    `q` the channel's share), constant on a channel that splits and
///    narrowing by [`TERMINAL_TAPER`] on one that does not; colour, order and
///    own order are the river's at the apex, discharge NaN (no reading), rank
///    [`delta_rank`] of [`RankOfDischarge`]'s for `q` times the river's
///    discharge, the apex's and `start`.
///
/// `river_width` is the preset's width multiplier (`river_width_factor`), used
/// only for the water cap; `start` is the map's `river_stroke::start_share`
/// (the zoom law's opening share), used only for [`delta_rank`].
/// Deterministic: a function of `geom`, the grid, `start` and `wet`. The
/// network itself is only read.
pub fn delta_fans(geom: &RiverGeometry, gw: usize, gh: usize, river_width: f32, start: f32, wet: Wet) -> (RiverGeometry, DeltaStats) {
    let mut st = DeltaStats::default();
    let mut out: Vec<DrawnRun> = Vec::new();
    let cells = (gw * gh) as f32;
    let s_min = gw as f32 / FIELD_VIEW_PX;
    let ranks = RankOfDischarge::of(geom);
    for run in &geom.runs {
        let n = run.pts.len();
        let Some(&(s0, e0)) = run.pieces.last() else { continue };
        let tail_meets_water = run.reach.get(run.pieces.len() - 1).is_some_and(|r| r.1.is_some());
        let e0 = e0.min(n);
        if !tail_meets_water || e0 < s0 + 2 || run.widths.len() != n || run.colors.len() != n || run.orders.len() != n || run.discharge.len() != n {
            continue;
        }
        st.mouths += 1;
        if run.own_order < MIN_ORDER {
            continue;
        }
        let q = run.discharge[s0..e0].iter().copied().filter(|v| v.is_finite()).fold(None, |m: Option<f32>, v| Some(m.map_or(v, |m| m.max(v))));
        let Some(q_river) = q else {
            st.no_discharge += 1;
            continue;
        };
        if q_river < MIN_DISCHARGE_FRAC * cells {
            continue;
        }
        st.eligible += 1;
        // 1. The shore crossing: the last dry point of the piece.
        let Some(kd) = (s0..e0).rev().find(|&k| !wet_at(wet, run.pts[k])) else {
            st.skipped += 1;
            continue;
        };
        let d = run.pts[kd];
        // 2. The apex, `u_want` cells of arc length upstream of it: point index
        // `ja` and the fraction `t` of the way from it toward `ja - 1`.
        let u_want = (run.own_order as f32 - 1.0) * SETBACK_PER_ORDER_CELLS;
        let (mut acc, mut j) = (0.0f32, kd);
        let mut apex: Option<(usize, f32)> = None;
        while j > s0 {
            let l = dist(run.pts[j], run.pts[j - 1]);
            if l > 0.0 && acc + l >= u_want {
                apex = Some((j, (u_want - acc) / l));
                acc = u_want;
                break;
            }
            acc += l;
            j -= 1;
        }
        let (a, ja) = match apex {
            Some((jj, t)) => {
                let (p, pp) = (run.pts[jj], run.pts[jj - 1]);
                ((p.0 + (pp.0 - p.0) * t, p.1 + (pp.1 - p.1) * t), jj)
            }
            // The piece was shorter than the setback: the fan starts at its head.
            None => (run.pts[s0], s0),
        };
        let ai = ja.saturating_sub(1).max(s0);
        let u_eff = acc.min(u_want);
        if u_eff < MIN_SETBACK_CELLS || wet_at(wet, a) {
            st.skipped += 1;
            continue;
        }
        let chord = (d.0 - a.0, d.1 - a.1);
        let cl = (chord.0 * chord.0 + chord.1 * chord.1).sqrt();
        // 1e-3 cells: labelled judgement -- a mouth chord shorter than this has no
        // direction to look out along, far below any real chord (the shortest kept
        // setback is MIN_SETBACK_CELLS).
        if cl < 1e-3 {
            st.skipped += 1;
            continue;
        }
        // 3. The plan type from the water in front of the mouth.
        let dir = (chord.0 / cl, chord.1 / cl);
        let kind = DeltaKind::of_exposure(exposure(d, dir, u_eff, wet).unwrap_or(0.0));
        // The river's own course from the apex to the shore: the delta's main
        // channel, never redrawn. Its land cells are occupied, so no
        // distributary crosses it.
        let mut course: Vec<(f32, f32)> = vec![a];
        course.extend(run.pts[ja..=kd].iter().copied());
        let mut cum = vec![0.0f32; course.len()];
        for k in 1..course.len() {
            cum[k] = cum[k - 1] + dist(course[k], course[k - 1]);
        }
        let mut g = Grow {
            wet,
            gw,
            gh,
            lmax: MAX_REACH_OVER_SETBACK * u_eff,
            kind,
            rng: Rng::for_mouth(a, run.own_order),
            occ: HashMap::new(),
            owned: Vec::new(),
            chans: Vec::new(),
            cap_scale: (river_width, s_min, 0.0),
            sea: SeaField::around(a, MAX_REACH_OVER_SETBACK * u_eff + 2.0, gw, gh, wet),
            course: Vec::new(),
            rejoined: 0,
            dropped_dry: 0,
            collided: 0,
            splits: 0,
        };
        for k in 0..course.len().saturating_sub(1) {
            // The course sampled every half cell, so a channel cannot slip
            // between two of its vertices.
            let steps = (dist(course[k], course[k + 1]) / STEP_CELLS).ceil().max(1.0) as usize;
            for t in 0..=steps {
                let f = t as f32 / steps as f32;
                let p = (course[k].0 + (course[k + 1].0 - course[k].0) * f, course[k].1 + (course[k + 1].1 - course[k].1) * f);
                if !wet_at(wet, p) {
                    g.occ.insert((p.0.floor() as i32, p.1.floor() as i32), u32::MAX);
                    g.course.push(p);
                }
            }
        }
        // 4. The offtakes, spread over the span with a drawn jitter of a
        // third of their spacing, alternating sides from a drawn first side.
        let n_off = kind.offtakes(run.own_order);
        let total = *cum.last().unwrap_or(&0.0);
        let mut side = if g.rng.unit() < 0.5 { 1.0f32 } else { -1.0 };
        let mut q_main = 1.0f32;
        let mut roots: Vec<usize> = Vec::new();
        let span = OFFTAKE_SPAN.1 - OFFTAKE_SPAN.0;
        for o in 0..n_off {
            let frac = OFFTAKE_SPAN.0 + span * (o as f32 + 0.5 + g.rng.range(-0.33, 0.33)) / n_off as f32;
            let at = frac * total;
            let k = cum.partition_point(|&c| c < at).clamp(1, course.len() - 1);
            let t = ((at - cum[k - 1]) / (cum[k] - cum[k - 1]).max(1e-6)).clamp(0.0, 1.0);
            let p = (course[k - 1].0 + (course[k].0 - course[k - 1].0) * t, course[k - 1].1 + (course[k].1 - course[k - 1].1) * t);
            // The course's local heading over a cell either side, so one kinked
            // vertex does not set it.
            let back = course[cum.partition_point(|&c| c < at - 1.0).min(course.len() - 1)];
            let fwd = course[cum.partition_point(|&c| c < at + 1.0).min(course.len() - 1)];
            let hc = (fwd.1 - back.1).atan2(fwd.0 - back.0);
            let share = g.rng.range(MINOR_SHARE.0, MINOR_SHARE.1);
            let full = (BIFURCATION_ANGLE_DEG + g.rng.range(-ANGLE_JITTER_DEG, ANGLE_JITTER_DEG)).to_radians();
            // The river's width at the offtake: its width at the course vertex.
            let w_ref = run.widths[(ja + k - 1).min(kd)];
            let q_off = q_main * share;
            match g.grow(p, hc + side * full * (1.0 - share), q_off, w_ref, at, 1) {
                Ok(c) => {
                    roots.push(c);
                    // A loop gives its water back to the river further down;
                    // the later offtakes' shares are of the river's remaining
                    // water, so a loop does not reduce it (a labelled
                    // simplification: where it rejoins is not tracked).
                    if !g.chans[c].rejoins {
                        q_main -= q_off;
                    }
                }
                Err(f) => g.count(f),
            }
            side = -side;
        }
        st.dropped_dry += g.dropped_dry;
        st.collided += g.collided;
        st.splits += g.splits;
        st.rejoined += g.rejoined;
        if roots.is_empty() {
            continue;
        }
        st.fans += 1;
        st.kinds[kind as usize] += 1;
        // 5. Every kept channel as a run, in growth order.
        let apex_rank = run.ranks.get(ai).copied().unwrap_or(0.0);
        let runs_before = out.len();
        let mut stack: Vec<usize> = roots.iter().rev().copied().collect();
        while let Some(c) = stack.pop() {
            let ch = &g.chans[c];
            if !ch.alive {
                continue;
            }
            stack.extend(ch.kids.iter().rev().copied());
            let w0 = ch.w_ref * ch.q.sqrt();
            // A loop joins the river at its own width; only a channel ending in
            // the water tapers.
            let taper = if ch.kids.is_empty() && !ch.rejoins { TERMINAL_TAPER } else { 0.0 };
            let len = ch.pts.windows(2).map(|s| dist(s[0], s[1])).sum::<f32>().max(1e-6);
            let mut at = 0.0f32;
            let mut widths = Vec::with_capacity(ch.pts.len());
            for (k, p) in ch.pts.iter().enumerate() {
                if k > 0 {
                    at += dist(*p, ch.pts[k - 1]);
                }
                widths.push(w0 * (1.0 - taper * (at / len)));
            }
            let m = ch.pts.len();
            // RZ-2 / RZ-5: never ahead of the apex river, ranked as a river of
            // its own discharge would be, and whole by x16 (section 12).
            let rank = delta_rank(ranks.rank(ch.q * q_river), apex_rank, start);
            out.push(DrawnRun {
                pts: ch.pts.clone(),
                widths,
                colors: vec![run.colors[ai]; m],
                orders: vec![run.orders[ai]; m],
                discharge: vec![f32::NAN; m],
                ranks: if run.ranks.is_empty() { Vec::new() } else { vec![rank; m] },
                pieces: vec![(0, m)],
                reach: vec![(None, None)],
                own_order: run.own_order,
            });
            st.branches += 1;
        }
        st.fan_sizes.push((out.len() - runs_before) as u32);
        st.fan_apex_rank.push(run.ranks.get(ai).copied().unwrap_or(f32::NAN));
    }
    (RiverGeometry { runs: out }, st)
}

/// **What of the fans a view shows** (the owner, 2026-11-11, on the x16
/// frames: *"the deltas seem to have gone"*; `DELTA_GEOMETRY_RESEARCH.md`
/// section 12), measured by the same law every raster applies.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FanSeen {
    /// Fans built.
    pub mouths: usize,
    /// Fans whose apex river is itself drawn at this view (visibility at
    /// least `PICK_MIN_VISIBILITY`); an unranked apex counts as drawn.
    pub river_shown: usize,
    /// Channels built, over all fans.
    pub generated: usize,
    /// Channels drawn at this view (some point at or above
    /// `PICK_MIN_VISIBILITY`, the "hidden cannot be picked" line).
    pub drawn: usize,
    /// Of the fans whose river is shown, those drawing at least one channel.
    pub shown_with_channel: usize,
    /// Median full drawn width, screen pixels, over the drawn channels'
    /// points (NaN when none is drawn).
    pub width_px_p50: f32,
    /// Tenth percentile of the same widths (NaN when none is drawn).
    pub width_px_p10: f32,
    /// Median visibility rank of the channels (first point; NaN when the fans
    /// are unranked). Zoom-independent: the diagnosis of section 12.
    pub rank_p50: f32,
    /// Median apex-river rank over the fans (NaN when unranked).
    pub apex_rank_p50: f32,
}

/// [`FanSeen`] of `fans` (split into mouths by `st.fan_sizes`) at `ppc`
/// pixels per cell, with the law's scale given (`fit`, `start`) and the
/// preset's river width `rw`. The width is the field's own rule
/// (`river_field::edge_px`): ground width times `ppc`, the headwater
/// narrowing and `rw`, floored at [`MIN_STROKE_PX`]. A measurement only;
/// nothing draws from it.
pub fn fan_seen(fans: &RiverGeometry, st: &DeltaStats, ppc: f32, fit: f32, start: f32, rw: f32) -> FanSeen {
    use crate::river_stroke::{headwater_deemphasis, headwater_weight, river_visibility_at, PICK_MIN_VISIBILITY};
    let mut out = FanSeen { mouths: st.fan_sizes.len(), ..FanSeen::default() };
    let mut widths: Vec<f32> = Vec::new();
    let mut ranks: Vec<f32> = fans.runs.iter().filter_map(|r| r.ranks.first().copied()).filter(|u| u.is_finite()).collect();
    let mut apexes: Vec<f32> = st.fan_apex_rank.iter().copied().filter(|u| u.is_finite()).collect();
    let p50 = |v: &mut Vec<f32>| {
        v.sort_by(f32::total_cmp);
        v.get(v.len() / 2).copied().unwrap_or(f32::NAN)
    };
    out.rank_p50 = p50(&mut ranks);
    out.apex_rank_p50 = p50(&mut apexes);
    let mut k0 = 0usize;
    for (f, &n) in st.fan_sizes.iter().enumerate() {
        let runs = &fans.runs[k0.min(fans.runs.len())..(k0 + n as usize).min(fans.runs.len())];
        k0 += n as usize;
        let apex = st.fan_apex_rank.get(f).copied().unwrap_or(f32::NAN);
        let shown = river_visibility_at(apex, ppc, fit, start) >= PICK_MIN_VISIBILITY;
        out.river_shown += shown as usize;
        let mut any = false;
        for r in runs {
            out.generated += 1;
            let mut drawn = false;
            for (k, w) in r.widths.iter().enumerate() {
                let u = r.ranks.get(k).copied().unwrap_or(f32::NAN);
                if river_visibility_at(u, ppc, fit, start) >= PICK_MIN_VISIBILITY {
                    drawn = true;
                    let wm = 1.0 + (headwater_deemphasis(ppc) - 1.0) * headwater_weight(u);
                    widths.push((w * ppc * wm * rw).max(MIN_STROKE_PX));
                }
            }
            out.drawn += drawn as usize;
            any |= drawn;
        }
        out.shown_with_channel += (shown && any) as usize;
    }
    widths.sort_by(f32::total_cmp);
    out.width_px_p50 = widths.get(widths.len() / 2).copied().unwrap_or(f32::NAN);
    out.width_px_p10 = widths.get(widths.len() / 10).copied().unwrap_or(f32::NAN);
    out
}

/// **The shape of a set of fans**, for the probe and the bars
/// (`DELTA_GEOMETRY_RESEARCH.md` section 6): what the eye reads as natural or
/// as a comb, measured on the drawn runs alone.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DeltaShape {
    /// Channels measured (runs of at least [`SHAPE_MIN_CELLS`] on land).
    pub channels: usize,
    /// Sinuosity (path over chord, on land) of those channels: the 10th,
    /// 50th and 90th percentiles.
    pub sinuosity: [f32; 3],
    /// Share of the fans' land length that runs within two cells of ANOTHER
    /// fan channel and within [`PARALLEL_DEG`] of its heading -- the comb.
    pub parallel_frac: f32,
    /// Bifurcations found (two runs starting at one point): their count, the
    /// mean and standard deviation of the full angle between the daughters'
    /// first [`SHAPE_MIN_CELLS`] cells, and the mean minor share by `w^2`.
    pub splits: usize,
    /// See [`Self::splits`].
    pub split_angle_mean: f32,
    /// See [`Self::splits`].
    pub split_angle_sd: f32,
    /// See [`Self::splits`].
    pub minor_share_mean: f32,
}

/// The least land length, in cells, a channel needs to be measured by
/// [`shape_of`]. **Labelled judgement**: under three cells a chord is too short
/// for sinuosity or heading to mean anything.
pub const SHAPE_MIN_CELLS: f32 = 3.0;

/// Two channel pieces closer than two cells whose headings differ by less
/// than this are "parallel" ([`DeltaShape::parallel_frac`]). **Labelled
/// judgement**: the old fan's inner branch left the river at 11 degrees.
pub const PARALLEL_DEG: f32 = 15.0;

/// Measures [`DeltaShape`] over `fans` (one run per channel), on land by
/// `wet`. Pure; read only by the probe and the tests.
pub fn shape_of(fans: &RiverGeometry, wet: Wet) -> DeltaShape {
    let land: Vec<Vec<(f32, f32)>> = fans.runs.iter().map(|r| r.pts.iter().copied().take_while(|&p| !wet_at(wet, p)).collect()).collect();
    let len = |v: &[(f32, f32)]| v.windows(2).map(|s| dist(s[0], s[1])).sum::<f32>();
    let mut sins: Vec<f32> = Vec::new();
    for v in &land {
        let l = len(v);
        if l >= SHAPE_MIN_CELLS && v.len() >= 2 {
            sins.push(l / dist(v[0], *v.last().unwrap()).max(1e-6));
        }
    }
    sins.sort_by(f32::total_cmp);
    let pct = |p: f32| if sins.is_empty() { f32::NAN } else { sins[((sins.len() - 1) as f32 * p).round() as usize] };
    // Parallel length: every land segment against every other run's segments.
    let segs: Vec<(usize, (f32, f32), f32, f32)> = land
        .iter()
        .enumerate()
        .flat_map(|(i, v)| v.windows(2).map(move |s| (i, ((s[0].0 + s[1].0) * 0.5, (s[0].1 + s[1].1) * 0.5), (s[1].1 - s[0].1).atan2(s[1].0 - s[0].0), dist(s[0], s[1]))))
        .collect();
    // Bucketed by two-cell squares, so the neighbours of a segment are the
    // nine buckets round it (a whole world's fans are ~10^4 segments).
    let bucket = |m: (f32, f32)| ((m.0 / 2.0).floor() as i32, (m.1 / 2.0).floor() as i32);
    let mut grid: HashMap<(i32, i32), Vec<usize>> = HashMap::new();
    for (k, sg) in segs.iter().enumerate() {
        grid.entry(bucket(sg.1)).or_default().push(k);
    }
    let (mut total, mut par) = (0.0f32, 0.0f32);
    for &(i, m, h, l) in &segs {
        total += l;
        let (bx, by) = bucket(m);
        let near = (-1..=1).flat_map(|dy| (-1..=1).map(move |dx| (bx + dx, by + dy))).filter_map(|b| grid.get(&b)).flatten().map(|&k| segs[k]);
        let hit = near.into_iter().any(|(j, m2, h2, _)| {
            let dh = (h - h2).rem_euclid(std::f32::consts::PI);
            let dh = dh.min(std::f32::consts::PI - dh);
            j != i && dist(m, m2) < 2.0 && dh < PARALLEL_DEG.to_radians() && dist(m, land[i][0]) > EXEMPT_CELLS && dist(m2, land[j][0]) > EXEMPT_CELLS
        });
        if hit {
            par += l;
        }
    }
    // Splits: pairs of runs starting at one point.
    let heading_over = |v: &[(f32, f32)]| {
        let e = v.iter().find(|&&p| dist(p, v[0]) >= SHAPE_MIN_CELLS).copied().unwrap_or(*v.last().unwrap());
        (e.1 - v[0].1).atan2(e.0 - v[0].0)
    };
    let mut by_start: HashMap<(u32, u32), Vec<usize>> = HashMap::new();
    for (i, r) in fans.runs.iter().enumerate() {
        if r.pts.len() >= 2 {
            by_start.entry((r.pts[0].0.to_bits(), r.pts[0].1.to_bits())).or_default().push(i);
        }
    }
    let (mut angles, mut shares) = (Vec::new(), Vec::new());
    let mut keys: Vec<_> = by_start.keys().copied().collect();
    keys.sort();
    for k in keys {
        let v = &by_start[&k];
        if v.len() != 2 {
            continue;
        }
        let (a, b) = (&fans.runs[v[0]], &fans.runs[v[1]]);
        let d = (heading_over(&a.pts) - heading_over(&b.pts)).rem_euclid(std::f32::consts::TAU);
        angles.push(d.min(std::f32::consts::TAU - d).to_degrees());
        let (wa, wb) = (a.widths[0].powi(2), b.widths[0].powi(2));
        shares.push(wa.min(wb) / (wa + wb).max(1e-12));
    }
    let mean = |v: &[f32]| if v.is_empty() { f32::NAN } else { v.iter().sum::<f32>() / v.len() as f32 };
    let am = mean(&angles);
    let sd = if angles.is_empty() { f32::NAN } else { (angles.iter().map(|x| (x - am).powi(2)).sum::<f32>() / angles.len() as f32).sqrt() };
    DeltaShape {
        channels: sins.len(),
        sinuosity: [pct(0.1), pct(0.5), pct(0.9)],
        parallel_frac: if total > 0.0 { par / total } else { 0.0 },
        splits: angles.len(),
        split_angle_mean: am,
        split_angle_sd: sd,
        minor_share_mean: mean(&shares),
    }
}

/// **RIM-7: whether a look draws delta fans at all** -- the one gate the
/// painted screen path (`WorldGen::build_color_texture`), the deep-zoom tiles
/// (`WorldGen::lod_snapshot_inputs`) and the export
/// (`WorldGen::export_river_fans`) share, so no zoom and no export can
/// show a fan the screen does not, or drop one it does.
///
/// True when the off switch is clear (`off` is `WorldGen::set_river_deltas`
/// negated) and the look paints its rivers into the map -- `smooth_shores`
/// and `rivers_as_water`, the painted path's own condition -- on a grid whose
/// river field fits its texel budget (`river_field::field_scale`): the
/// painted-river gate itself, `river_field::painted`, which since RIM-7's
/// second part also decides whether the tiles and the export paint the
/// network or stroke it. Each other
/// case is a base view that draws the vector stroke (`map_overlay.gd`), which
/// draws no fan, so the tiles and the export of that look must not either:
/// a fan appearing only past the deep-zoom switch would be the same pop this
/// gate exists to remove, the other way round.
///
/// Must never read anything but its arguments: the three callers pass the
/// appearance they draw with (an export its own style override), and the
/// answer must be the same function of it everywhere.
pub fn fans_drawn(a: &crate::render::TerrainAppearance, off: bool, gw: usize, gh: usize) -> bool {
    !off && crate::river_field::painted(a, gw, gh)
}

/// Tests of the delta fans: the gate that decides which mouths get one, the
/// plan type, the tree's shape (unequal splits, the bifurcation angle, width
/// conservation, sinuosity, no comb), the rank, the off switch and its
/// bit-identical field. Fixtures are straight east-heading runs on a 100 x 100
/// grid whose water is to the east.
#[cfg(test)]
mod tests {
    use super::*;
    use crate::river_stroke::ShoreReach;

    /// Protects: Ruling BW item 3's guarantee that comb thinning never hides a
    /// delta-fan mouth -- the draw plan's never-thin discharge share
    /// (`cartalith_hydrology::COMB_DISCHARGE_FRAC`) must not exceed the share a
    /// mouth needs for a fan ([`MIN_DISCHARGE_FRAC`]), and both are pinned to
    /// the literal. Mutation: raising the hydrology constant above this one
    /// fails the first assert; changing either value fails the second.
    #[test]
    fn comb_thinning_never_hides_a_fan_mouth() {
        assert!(cartalith_hydrology::COMB_DISCHARGE_FRAC <= MIN_DISCHARGE_FRAC);
        assert_eq!((cartalith_hydrology::COMB_DISCHARGE_FRAC, MIN_DISCHARGE_FRAC), (2.0e-4, 2.0e-4));
        // A fan needs order >= MIN_ORDER (> the comb cap's order of 1).
        assert!(MIN_ORDER > cartalith_hydrology::COMB_MAX_ORDER);
    }

    /// The whole-world start share (`river_stroke::start_share(40 075 km)`,
    /// 0.044) the fixtures rank their fans at.
    const WORLD_START: f32 = 0.044;

    // Grid width of every fixture: a judgement -- large enough that a fan (a few cells) and the
    // `MIN_DISCHARGE_FRAC` share (2e-4 of cells) are both representable, small enough to be fast.
    const GW: usize = 100;
    // Grid height of the fixtures; see `GW`.
    const GH: usize = 100;

    /// A run heading east along `y = 50.5` from `x = 10.5` to `x_end`, one
    /// point a cell, whose last piece ends meeting water (`reach` tail set),
    /// of order `order` and discharge `q` everywhere. Full width 1 cell.
    fn run(order: i16, q: f32, x_end: f32, tail: bool) -> DrawnRun {
        run_at(50.5, order, q, x_end, tail)
    }

    /// [`run`] along `y` instead of 50.5.
    fn run_at(y: f32, order: i16, q: f32, x_end: f32, tail: bool) -> DrawnRun {
        let n = (x_end - 10.5) as usize + 1;
        DrawnRun {
            pts: (0..n).map(|i| (10.5 + i as f32, y)).collect(),
            widths: vec![1.0; n],
            colors: vec![[0.2, 0.4, 0.8, 1.0]; n],
            orders: vec![order; n],
            discharge: vec![q; n],
            pieces: vec![(0, n)],
            reach: vec![(None, tail.then(ShoreReach::default))],
            ranks: Vec::new(),
            own_order: order,
        }
    }

    /// [`run`] starting at `x0` instead of 10.5.
    fn run_from(x0: f32, order: i16, q: f32, x_end: f32) -> DrawnRun {
        let mut r = run(order, q, x_end, true);
        let skip = (x0 - 10.5) as usize;
        r.pts.drain(..skip);
        r.widths.drain(..skip);
        r.colors.drain(..skip);
        r.orders.drain(..skip);
        r.discharge.drain(..skip);
        r.pieces = vec![(0, r.pts.len())];
        r
    }

    /// An open coast: every cell with `x >= 60` is sea (exposure 1, wave-dominated).
    fn east_coast(x: i64, _y: i64) -> bool {
        x >= 60
    }

    /// A long narrow lake six cells wide, `60 <= x < 66`, land beyond:
    /// sheltered water (river-dominated).
    fn narrow_lake(x: i64, _y: i64) -> bool {
        (60..66).contains(&x)
    }

    /// A lake twelve cells wide, `60 <= x < 72`: between the two (lobate).
    fn wide_lake(x: i64, _y: i64) -> bool {
        (60..72).contains(&x)
    }

    /// The three fixture coasts with their names.
    fn coasts() -> [(&'static str, Wet<'static>); 3] {
        [("open", &east_coast), ("narrow lake", &narrow_lake), ("wide lake", &wide_lake)]
    }

    /// Runs [`delta_fans`] on a single run over the fixture grid (cell scale 1.0) with
    /// the given wetness predicate; returns the fans and their stats.
    fn fans_of(r: DrawnRun, wet: Wet) -> (RiverGeometry, DeltaStats) {
        delta_fans(&RiverGeometry { runs: vec![r] }, GW, GH, 1.0, WORLD_START, wet)
    }

    /// Whether run `r` of `fan` is a terminal channel: no other run starts at
    /// its last point (a channel that split ends where its daughters start).
    fn is_leaf(fan: &RiverGeometry, r: &DrawnRun) -> bool {
        let e = *r.pts.last().unwrap();
        !fan.runs.iter().any(|o| o.pts[0] == e)
    }

    /// Whether point `e` lies on the polyline of `river` (within 1e-3 cells):
    /// where a loop ends (section 12).
    fn on_course(river: &DrawnRun, e: (f32, f32)) -> bool {
        river.pts.windows(2).any(|w| {
            let (a, b) = (w[0], w[1]);
            let (vx, vy) = (b.0 - a.0, b.1 - a.1);
            let t = (((e.0 - a.0) * vx + (e.1 - a.1) * vy) / (vx * vx + vy * vy).max(1e-12)).clamp(0.0, 1.0);
            dist(e, (a.0 + vx * t, a.1 + vy * t)) < 1e-3
        })
    }

    /// Protects: the plan type follows the water in front of the mouth, with
    /// the thresholds as LITERALS: the open coast's half-disc is all water
    /// (wave-dominated), a six-cell lake about a third (river-dominated) and a
    /// twelve-cell one about 0.6 (lobate); and the stats count each fan's kind.
    /// Mutating either threshold, or the half-disc's facing, fails here.
    #[test]
    fn the_plan_type_follows_the_mouths_exposure() {
        let d = (59.5, 50.5);
        let e = |w: Wet| exposure(d, (1.0, 0.0), 24.0, w).unwrap();
        assert!(e(&east_coast) > 0.97, "{}", e(&east_coast));
        assert!((0.25..0.40).contains(&e(&narrow_lake)), "{}", e(&narrow_lake));
        assert!((0.55..0.70).contains(&e(&wide_lake)), "{}", e(&wide_lake));
        // Facing away from the water: a river flowing west would see none of it.
        assert!(exposure(d, (-1.0, 0.0), 24.0, &east_coast).unwrap() < 0.03);
        for (x, k) in [(0.54, DeltaKind::RiverDominated), (0.56, DeltaKind::Lobate), (0.79, DeltaKind::Lobate), (0.81, DeltaKind::WaveDominated)] {
            assert_eq!(DeltaKind::of_exposure(x), k, "exposure {x}");
        }
        for (name, w, slot) in [("open", &east_coast as Wet, 2usize), ("narrow", &narrow_lake, 0), ("wide", &wide_lake, 1)] {
            let (_, st) = fans_of(run(4, 5000.0, 62.5, true), w);
            assert_eq!(st.kinds[slot], 1, "{name}: {st:?}");
            assert_eq!(st.kinds.iter().sum::<usize>(), st.fans, "{name}: {st:?}");
        }
    }

    /// Protects: every fan exists and stays in bounds -- each eligible mouth on
    /// each coast gets one; every channel starts on the river's own course
    /// between the apex (`(own_order - 1) * SETBACK_PER_ORDER_CELLS` upstream
    /// of the last dry point) and the shore, ends ON water or ON the river's
    /// own course (a loop, section 12), and never lies on
    /// land farther than `MAX_REACH_OVER_SETBACK` setbacks from the apex; it
    /// carries no discharge reading and the river's colour and own order.
    #[test]
    fn every_fan_starts_on_the_river_and_ends_on_water() {
        for (name, w) in coasts() {
            for order in [3i16, 4, 5] {
                let river = run(order, 5000.0, 62.5, true);
                let (fan, st) = fans_of(run(order, 5000.0, 62.5, true), w);
                assert_eq!((st.mouths, st.eligible, st.fans), (1, 1, 1), "{name} order {order}: {st:?}");
                assert_eq!(fan.runs.len(), st.branches);
                let setback = (order as f32 - 1.0) * SETBACK_PER_ORDER_CELLS;
                let apex = (59.5 - setback, 50.5);
                for r in &fan.runs {
                    let (a, e) = (r.pts[0], *r.pts.last().unwrap());
                    assert!(a.0 >= apex.0 - 1e-3 && a.0 < 60.0, "{name} order {order}: start {a:?}");
                    assert!(!is_leaf(&fan, r) || wet_at(w, e) || on_course(&river, e), "{name}: a terminal channel ends on water or on the river (a loop): {e:?}");
                    for p in &r.pts {
                        assert!(wet_at(w, *p) || dist(*p, apex) <= MAX_REACH_OVER_SETBACK * setback + 1e-3, "{name}: land point {p:?} too far from the apex");
                    }
                    assert_eq!(r.own_order, order);
                    assert!(r.discharge.iter().all(|v| v.is_nan()), "a fan has no discharge reading, and must never fake one");
                    assert_eq!(r.colors[0], [0.2, 0.4, 0.8, 1.0]);
                }
                // The roots (channels leaving the river) start ON its line.
                assert!(fan.runs.iter().any(|r| (r.pts[0].1 - 50.5).abs() < 1e-4), "{name}: no channel leaves the river itself");
            }
        }
    }

    /// Protects: a sheltered mouth grows a branching tree and an open coast
    /// few channels -- the river-dominated narrow lake splits (both daughters
    /// kept at least once) and has more channels than the wave-dominated open
    /// coast, whose distributaries never split (`generations` 1).
    #[test]
    fn a_sheltered_mouth_branches_more_than_an_open_one() {
        for order in [4i16, 5] {
            let (open, so) = fans_of(run(order, 5000.0, 62.5, true), &east_coast);
            let (lake, sl) = fans_of(run(order, 5000.0, 62.5, true), &narrow_lake);
            assert_eq!(so.splits, 0, "order {order}: {so:?}");
            assert!(sl.splits >= 1 && lake.runs.len() > open.runs.len(), "order {order}: lake {sl:?}, open {so:?}");
        }
    }

    /// The kept bifurcations of a fan: for every pair of runs starting at one
    /// point, `(parent's width there, the two daughters' start widths)`. The
    /// parent is the run whose last land point is that point.
    fn splits_of(fan: &RiverGeometry) -> Vec<(f32, f32, f32)> {
        let mut out = Vec::new();
        for (i, a) in fan.runs.iter().enumerate() {
            for b in &fan.runs[i + 1..] {
                if a.pts[0] == b.pts[0] {
                    let parent = fan.runs.iter().find_map(|r| r.pts.iter().position(|&p| p == a.pts[0]).filter(|&k| k > 0).map(|k| r.widths[k]));
                    if let Some(wp) = parent {
                        out.push((wp, a.widths[0], b.widths[0]));
                    }
                }
            }
        }
        out
    }

    /// Protects: hydraulic geometry and unequal splits. At every kept
    /// bifurcation the daughters' squared widths sum to the parent's (`w ~
    /// sqrt(Q)` with the discharge conserved), so the width SUM is between 1
    /// and 1.42 times the parent's, and the minor daughter carries between 0.2
    /// and 0.45 of it -- never an equal split. Literals, not the constants.
    #[test]
    fn splits_conserve_discharge_and_are_unequal() {
        let mut n = 0;
        for y in [40.5f32, 45.5, 50.5, 55.5] {
            for order in [4i16, 5] {
                let (fan, _) = fans_of(run_at(y, order, 5000.0, 62.5, true), &narrow_lake);
                for (wp, wa, wb) in splits_of(&fan) {
                    n += 1;
                    let sq = (wa * wa + wb * wb) / (wp * wp);
                    assert!((sq - 1.0).abs() < 1e-3, "w^2 not conserved: {wp} -> {wa} + {wb}");
                    let sum = (wa + wb) / wp;
                    assert!((1.0..1.42).contains(&sum), "width sum {sum}");
                    let minor = (wa * wa).min(wb * wb) / (wp * wp);
                    assert!((0.2 - 1e-3..=0.45 + 1e-3).contains(&minor), "minor share {minor}");
                }
            }
        }
        assert!(n >= 4, "the fixtures reached only {n} splits");
    }

    /// Protects: the bifurcation angle -- over every kept split of the
    /// fixtures the full angle between the daughters (over their first three
    /// cells, `shape_of`) averages about 70 degrees (Coffey & Shaw 2017) and
    /// spreads (no regular fan); the old fixed +-22/45 pairs read 22 or 45.
    #[test]
    fn daughters_part_at_about_seventy_degrees_and_not_regularly() {
        let mut angles = Vec::new();
        for y in [40.5f32, 45.5, 50.5, 55.5, 60.5] {
            for order in [4i16, 5] {
                let (fan, _) = fans_of(run_at(y, order, 5000.0, 62.5, true), &narrow_lake);
                let sh = shape_of(&fan, &narrow_lake);
                if sh.splits > 0 {
                    angles.push((sh.split_angle_mean, sh.splits));
                }
            }
        }
        let n: usize = angles.iter().map(|a| a.1).sum();
        let mean = angles.iter().map(|a| a.0 * a.1 as f32).sum::<f32>() / n as f32;
        assert!(n >= 4 && (60.0..80.0).contains(&mean), "{n} splits, mean {mean}: {angles:?}");
        let (lo, hi) = angles.iter().fold((f32::MAX, f32::MIN), |m, a| (m.0.min(a.0), m.1.max(a.0)));
        assert!(hi - lo > 4.0, "every split at one angle: {angles:?}");
    }

    /// Protects: the channels meander and never form a comb. On every coast
    /// the median sinuosity of the fan's channels is a low distributary's (over
    /// 1.01 -- the old straight branches were 1.000 -- and under 1.2), and no
    /// channel runs within two cells of another and within 15 degrees of its
    /// heading for more than 5 % of the fan's land length; and no two channels
    /// share a cell's neighbourhood away from their starts.
    #[test]
    fn channels_meander_and_never_run_side_by_side() {
        for (name, w) in coasts() {
            for y in [45.5f32, 50.5, 55.5] {
                let (fan, _) = fans_of(run_at(y, 5, 5000.0, 62.5, true), w);
                let sh = shape_of(&fan, w);
                assert!(sh.channels >= 1, "{name}");
                assert!(sh.sinuosity[1] > 1.01 && sh.sinuosity[1] < 1.2, "{name} y {y}: {sh:?}");
                assert!(sh.parallel_frac < 0.05, "{name} y {y}: a comb, {sh:?}");
                for (i, a) in fan.runs.iter().enumerate() {
                    for b in &fan.runs[i + 1..] {
                        for p in a.pts.iter().filter(|&&p| !wet_at(w, p) && dist(p, a.pts[0]) > EXEMPT_CELLS + 1.0) {
                            for q in b.pts.iter().filter(|&&q| !wet_at(w, q) && dist(q, b.pts[0]) > EXEMPT_CELLS + 1.0) {
                                let (di, dj) = ((p.0.floor() - q.0.floor()).abs(), (p.1.floor() - q.1.floor()).abs());
                                assert!(di > 1.0 || dj > 1.0, "{name}: channels touch at {p:?} / {q:?}");
                            }
                        }
                    }
                }
            }
        }
    }

    /// Protects: section 12's loops. On the narrow-lake fixture (an order-5
    /// river, the measured case) a distributary meets the river's own course;
    /// it is kept as a loop (`rejoined` 1), ending ON the course at least
    /// `LOOP_MIN_CELLS` (6, a literal) in a straight line from where it left,
    /// having run at least that far, and at its full width (no taper).
    /// Mutation: a `LOOP_MIN_CELLS` past the fixture's reach drops it again.
    #[test]
    fn a_channel_that_meets_the_river_rejoins_it_as_a_loop() {
        let river = run(5, 5000.0, 62.5, true);
        let (fan, st) = fans_of(run(5, 5000.0, 62.5, true), &narrow_lake);
        assert_eq!(st.rejoined, 1, "{st:?}");
        let loops: Vec<&DrawnRun> = fan.runs.iter().filter(|r| on_course(&river, *r.pts.last().unwrap())).collect();
        assert_eq!(loops.len(), 1);
        let l = loops[0];
        let (a, e) = (l.pts[0], *l.pts.last().unwrap());
        assert!(dist(a, e) >= 6.0, "the island is at least six cells: {a:?} -> {e:?}");
        let len: f32 = l.pts.windows(2).map(|w| dist(w[0], w[1])).sum();
        assert!(len >= 6.0, "{len}");
        assert_eq!(l.widths.first(), l.widths.last(), "a loop keeps its width to the river");
    }

    /// Protects: the comb measure sees a comb -- the old fan's failure. Two
    /// straight parallel runs one cell apart read `parallel_frac` near 1, two
    /// runs 30 degrees apart read 0 past their shared start, and a straight
    /// run's sinuosity is 1.
    #[test]
    fn the_shape_measure_sees_a_comb() {
        let line = |y: f32, dy: f32| DrawnRun { pts: (0..20).map(|i| (10.5 + i as f32, y + dy * i as f32)).collect(), widths: vec![0.5; 20], ..run(3, 1.0, 29.5, false) };
        let comb = RiverGeometry { runs: vec![line(50.5, 0.0), line(51.5, 0.0)] };
        let sh = shape_of(&comb, &|_, _| false);
        assert!(sh.parallel_frac > 0.8, "{sh:?}");
        assert!((sh.sinuosity[1] - 1.0).abs() < 1e-4, "{sh:?}");
        let apart = RiverGeometry { runs: vec![line(50.5, 0.0), line(50.5, 0.577)] };
        assert!(shape_of(&apart, &|_, _| false).parallel_frac < 0.05);
    }

    /// Protects: distributaries do not cross the river itself. Every land
    /// point of every channel more than `EXEMPT_CELLS` from its start lies
    /// outside the 3 x 3 cells round the river's own course (`y = 50.5`) --
    /// except a loop's last two cells, where it rejoins the course (section
    /// 12): a loop ENDS on the river and never crosses it.
    #[test]
    fn no_channel_runs_along_or_across_the_main_channel() {
        for (name, w) in coasts() {
            let river = run(5, 5000.0, 62.5, true);
            let (fan, _) = fans_of(run(5, 5000.0, 62.5, true), w);
            for r in &fan.runs {
                let e = *r.pts.last().unwrap();
                let rejoin = on_course(&river, e);
                for p in r.pts.iter().filter(|&&p| !wet_at(w, p) && dist(p, r.pts[0]) > EXEMPT_CELLS && !(rejoin && dist(p, e) <= 2.0)) {
                    assert!((p.1.floor() - 50.0).abs() > 1.0 || p.0 >= 60.0, "{name}: {p:?} on the river's course");
                }
            }
        }
    }

    /// Protects: terminal channels taper and splitting ones do not -- a channel
    /// with no daughters narrows to 75 % of its start (`TERMINAL_TAPER` 0.25, a
    /// literal), a channel that splits or rejoins the river keeps its width; widths never rise along
    /// a channel and never reach zero.
    #[test]
    fn terminal_channels_taper_and_parents_keep_their_width() {
        let river = run(5, 5000.0, 62.5, true);
        let (fan, st) = fans_of(run(5, 5000.0, 62.5, true), &narrow_lake);
        assert!(st.splits >= 1, "{st:?}");
        let starts: Vec<(f32, f32)> = fan.runs.iter().map(|r| r.pts[0]).collect();
        for r in &fan.runs {
            let w0 = r.widths[0];
            let tip = *r.widths.last().unwrap();
            let parent = r.pts[1..].iter().any(|p| starts.contains(p));
            // A loop (section 12) joins the river at its own width, like a parent.
            if parent || on_course(&river, *r.pts.last().unwrap()) {
                assert!((tip - w0).abs() < 1e-6, "a splitting channel keeps its width: {w0} -> {tip}");
            } else {
                assert!((tip / w0 - 0.75).abs() < 1e-3 && tip > 0.0, "tip {tip} of {w0}");
            }
            assert!(r.widths.windows(2).all(|v| v[1] <= v[0] + 1e-6));
        }
    }

    /// Protects: a river that is not a big mouth gets no fan -- order 2
    /// (`MIN_ORDER`), a Strahler-3 stub with a trickle of discharge
    /// (`MIN_DISCHARGE_FRAC`), a run whose tail meets no water (a confluence
    /// or an inland end) and a run whose only shore end is an INLET with the
    /// run going on (a through-lake river) -- and a missing reading is counted,
    /// not read as zero.
    #[test]
    fn only_a_large_river_that_ends_in_water_gets_a_fan() {
        let none = |r: DrawnRun| fans_of(r, &east_coast);
        let (f, st) = none(run(2, 5000.0, 62.5, true));
        assert!(f.runs.is_empty() && st.mouths == 1 && st.eligible == 0, "{st:?}");
        let (f, st) = none(run(3, MIN_DISCHARGE_FRAC * (GW * GH) as f32 * 0.9, 62.5, true));
        assert!(f.runs.is_empty() && st.eligible == 0, "{st:?}");
        let (f, st) = none(run(3, MIN_DISCHARGE_FRAC * (GW * GH) as f32 * 1.1, 62.5, true));
        assert!(!f.runs.is_empty() && st.eligible == 1, "just above the floor gets one: {st:?}");
        let (f, st) = none(run(5, 9e9, 55.5, false));
        assert!(f.runs.is_empty() && st.mouths == 0, "a run that ends on land is no mouth: {st:?}");
        // A through-lake river: first piece ends at an inlet (tail set), the
        // second piece ends inland.
        let mut thru = run(5, 9e9, 62.5, false);
        let n = thru.pts.len();
        thru.pieces = vec![(0, n / 2), (n / 2, n)];
        thru.reach = vec![(None, Some(ShoreReach::default())), (None, None)];
        let (f, st) = none(thru);
        assert!(f.runs.is_empty() && st.mouths == 0, "an inlet is not a mouth: {st:?}");
        let mut nodis = run(5, 0.0, 62.5, true);
        nodis.discharge = vec![f32::NAN; nodis.pts.len()];
        let (f, st) = none(nodis);
        assert!(f.runs.is_empty() && st.no_discharge == 1 && st.eligible == 0, "{st:?}");
    }

    /// Protects: `MIN_SETBACK_CELLS` and the short-piece rule -- an order-4
    /// river whose last piece is under the minimum above its mouth gets no
    /// fan (it is counted skipped, not drawn as a stub), one just over it gets
    /// a fan whose channels leave the river between the piece's head and the
    /// shore rather than the full setback up.
    #[test]
    fn a_river_too_short_above_its_mouth_gets_no_fan() {
        // The last dry point is x = 59.5: a run from 56.5 is 3 cells above it,
        // one from 51.5 is 8.
        let short = run_from(56.5, 4, 5000.0, 62.5);
        let (f, st) = fans_of(short, &east_coast);
        assert!(f.runs.is_empty() && st.skipped == 1, "3 cells above the mouth is under the minimum: {st:?}");
        let (f, st) = fans_of(run_from(51.5, 4, 5000.0, 62.5), &narrow_lake);
        assert_eq!(st.fans, 1, "{st:?}");
        assert!(f.runs.iter().all(|r| r.pts[0].0 >= 51.5 - 1e-3), "{:?}", f.runs.iter().map(|r| r.pts[0]).collect::<Vec<_>>());
    }

    /// Protects: no leak along a shore. With the water only on one side of
    /// the river's line (a coast running along it, the sea to the south), a
    /// channel heading north has no water to find and is DROPPED, not drawn
    /// to the limit; every kept channel reaches water within the reach limit;
    /// and a world with no water draws nothing.
    #[test]
    fn a_channel_that_meets_no_water_is_dropped_not_drawn_along_the_shore() {
        let bay = |x: i64, y: i64| x >= 54 && y >= 52;
        let mut dropped = 0;
        for y in [48.5f32, 49.5, 50.5] {
            let river = run_at(y, 5, 5000.0, 62.5, true);
            let (fan, st) = fans_of(run_at(y, 5, 5000.0, 62.5, true), &bay);
            dropped += st.dropped_dry + st.collided;
            let setback = 4.0 * SETBACK_PER_ORDER_CELLS;
            for r in fan.runs.iter().filter(|r| is_leaf(&fan, r) && !on_course(&river, *r.pts.last().unwrap())) {
                let first_wet = r.pts.iter().position(|p| wet_at(&bay, *p)).expect("a kept terminal channel reaches water");
                assert!(first_wet >= 1, "a channel starts dry");
                let apex = (59.5 - setback, y);
                for p in &r.pts[..first_wet] {
                    assert!(dist(*p, apex) <= MAX_REACH_OVER_SETBACK * setback + 1e-3, "{p:?}");
                }
            }
        }
        assert!(dropped >= 1, "no channel was dropped on a one-sided coast");
        // A world with no water in reach at all: nothing drawn, nothing leaks.
        let (fan, st) = fans_of(run(4, 5000.0, 62.5, true), &|_, _| false);
        assert!(fan.runs.is_empty(), "no water, no fan: {st:?}");
    }

    /// Protects: RZ-5 through Ruling EN's law and section 12's two fixes, at
    /// LITERAL values. The network is the mouth (discharge 5000, 52 cells of
    /// steps), a river of 1500 (30 cells) and one of 100 (30 cells); 112 cells
    /// in all. The table is the share of drawn length carrying at least a
    /// discharge: 0 above 5000, 52/112 = 0.4643 down to 5000, 82/112 = 0.7321
    /// down to 1500, 1 below 100 -- and it never reads the network's ranks, so
    /// the small river's rank raised to 1 (as `hide_short_stubs` raises a
    /// stub's) changes nothing (the defect that hid every fan below x64). A
    /// channel takes `delta_rank`: its own discharge's rank, never ahead of
    /// the river's (0.1) and never later than the share at x16 for the map
    /// (start 0.044: 0.044^(1/3) = 0.3530). An unranked network gives
    /// unranked fans.
    #[test]
    fn a_channel_takes_the_rank_of_its_own_discharge_never_ahead_of_its_river_and_whole_by_x16() {
        let mut mouth = run(4, 5000.0, 62.5, true);
        mouth.ranks = vec![0.1; mouth.pts.len()];
        let mut mid = run_at(20.5, 3, 1500.0, 40.5, false);
        mid.ranks = vec![0.5; mid.pts.len()];
        let mut small = run_at(80.5, 1, 100.0, 40.5, false);
        small.ranks = vec![1.0; small.pts.len()];
        let g = RiverGeometry { runs: vec![mid, mouth, small] };
        let table = RankOfDischarge::of(&g);
        let r = |q: f32| table.rank(q).unwrap();
        assert_eq!(r(6000.0), 0.0);
        assert!((r(5000.0) - 0.464_286).abs() < 1e-5 && (r(2000.0) - 0.464_286).abs() < 1e-5, "{} {}", r(5000.0), r(2000.0));
        assert!((r(1500.0) - 0.732_143).abs() < 1e-5, "{}", r(1500.0));
        assert_eq!(r(50.0), 1.0);
        let mut small2 = run_at(80.5, 1, 100.0, 40.5, false);
        small2.ranks = vec![0.9; small2.pts.len()];
        let (mut mid2, mut mouth2) = (run_at(20.5, 3, 1500.0, 40.5, false), run(4, 5000.0, 62.5, true));
        mid2.ranks = vec![0.5; mid2.pts.len()];
        mouth2.ranks = vec![0.1; mouth2.pts.len()];
        let other = RankOfDischarge::of(&RiverGeometry { runs: vec![mid2, mouth2, small2] });
        assert_eq!(other.rank(1500.0), table.rank(1500.0), "the table reads discharge only, never a raised rank");
        // delta_rank at literals.
        assert!((delta_rank(Some(0.9), 0.1, 0.044) - 0.353_035).abs() < 1e-5, "{}", delta_rank(Some(0.9), 0.1, 0.044));
        assert_eq!(delta_rank(Some(0.05), 0.1, 0.044), 0.1, "never ahead of its river");
        assert_eq!(delta_rank(Some(0.2), 0.1, 0.044), 0.2, "its own discharge's rank between the two");
        assert_eq!(delta_rank(Some(0.9), 0.5, 0.044), 0.5, "a river later than x16 keeps its fan with it");
        assert_eq!(delta_rank(None, 0.3, 0.044), 0.3);
        let (fan, _) = delta_fans(&g, GW, GH, 1.0, WORLD_START, &east_coast);
        assert!(!fan.runs.is_empty());
        for f in &fan.runs {
            let u = f.ranks[0];
            assert!(f.ranks.iter().all(|&v| v == u));
            assert!((0.1..=0.353_04).contains(&u), "a channel ranks between its river and x16: {u}");
            // Every channel is fully drawn at x16 wherever its river is.
            let fit = 0.46f32;
            assert_eq!(crate::river_stroke::river_visibility_at(u, 16.0 * fit, fit, WORLD_START), 1.0);
        }
        let (fan, _) = fans_of(run(4, 5000.0, 62.5, true), &east_coast);
        assert!(fan.runs.iter().all(|r| r.ranks.is_empty()), "an unranked network's fan is unranked (drawn at every zoom)");
    }

    /// Protects: the tuned constants against a silent retune, by LITERAL
    /// values (every behavioural test above reads literals too, so a changed
    /// constant fails twice). Changing any constant must be done on purpose
    /// and re-judged on the screen.
    #[test]
    fn the_tuned_constants_are_pinned_by_literals_not_by_themselves() {
        assert_eq!((MIN_ORDER, SETBACK_PER_ORDER_CELLS, MIN_SETBACK_CELLS, MAX_REACH_OVER_SETBACK), (3, 8.0, 4.0, 1.6));
        assert_eq!((BIFURCATION_ANGLE_DEG, ANGLE_JITTER_DEG, MINOR_SHARE), (70.0, 12.0, (0.2, 0.45)));
        assert_eq!((BIFURCATION_LENGTH_WIDTHS, MIN_SEGMENT_CELLS, MEANDER_WAVELENGTH_WIDTHS, MIN_WAVELENGTH_CELLS), (10.0, 3.0, 11.0, 4.0));
        assert_eq!((TERMINAL_TAPER, CLEAR_CELLS, EXEMPT_CELLS, STEP_CELLS, VERTEX_EVERY_STEPS), (0.25, 1, 3.0, 0.5, 2));
        assert_eq!((OFFTAKE_SPAN, EXPOSURE_SHELTERED, EXPOSURE_OPEN, SEAWARD_TURN_RAD_PER_CELL), ((0.15, 0.75), 0.55, 0.8, 0.06));
        // The shape measure's own thresholds (a mutated PARALLEL_DEG of 0 reads every comb as 0).
        assert_eq!((PARALLEL_DEG, SHAPE_MIN_CELLS), (15.0, 3.0));
        // Section 12's two constants: loops from six cells, every channel whole by x16.
        assert_eq!((LOOP_MIN_CELLS, DELTA_FULL_ZOOM), (6.0, 16.0));
        let k = [DeltaKind::RiverDominated, DeltaKind::Lobate, DeltaKind::WaveDominated];
        assert_eq!(k.map(|k| (k.offtakes(4), k.offtakes(5), k.generations())), [(2, 3, 3), (2, 3, 2), (1, 2, 1)]);
        assert_eq!(k.map(|k| k.meander_rad()), [0.45, 0.45, 0.35]);
        // The reach limit, behaviourally: the run ends dry at x = 62.5, so the order-4
        // apex is at x = 38.5 (setback 24, limit 38.4 cells of path); a sea at x >= 66
        // is reached, one at x >= 80 (41.5 cells from the apex) cannot be.
        let wall = |cx: i64| fans_of(run(4, 5000.0, 62.5, true), &move |x: i64, _y: i64| x >= cx).1.fans;
        assert_eq!((wall(66), wall(80)), (1, 0), "the reach limit is 1.6 setbacks of path");
        // The discharge floor: 2.0 cells on this 10 000-cell grid.
        let at = |q: f32| fans_of(run(3, q, 62.5, true), &east_coast).1.eligible;
        assert_eq!((at(1.9), at(2.1)), (0, 1), "the floor is 2e-4 of the grid's cells");
    }

    /// Protects: determinism and purity -- two calls agree bit for bit, the
    /// network handed in is not touched, and the fan is seeded by where the
    /// mouth is: the same river one cell north draws a different tree (not the
    /// same one moved), so neighbouring deltas do not repeat.
    #[test]
    fn the_fans_are_a_pure_deterministic_function_of_the_network() {
        let g = RiverGeometry { runs: vec![run(5, 5000.0, 62.5, true)] };
        let before: Vec<(f32, f32)> = g.runs[0].pts.clone();
        let bits = |f: &RiverGeometry| f.runs.iter().flat_map(|r| r.pts.iter().map(|p| (p.0.to_bits(), p.1.to_bits())).chain(r.widths.iter().map(|w| (w.to_bits(), 0)))).collect::<Vec<_>>();
        let (a, sa) = delta_fans(&g, GW, GH, 1.0, WORLD_START, &narrow_lake);
        let (b, sb) = delta_fans(&g, GW, GH, 1.0, WORLD_START, &narrow_lake);
        // Compared as text: an unranked fan's apex rank is NaN ("no value"),
        // which `==` never equals.
        assert_eq!(format!("{sa:?}"), format!("{sb:?}"));
        assert_eq!(bits(&a), bits(&b));
        assert_eq!(g.runs[0].pts, before);
        let (c, _) = fans_of(run_at(51.5, 5, 5000.0, 62.5, true), &narrow_lake);
        let shifted: Vec<_> = c.runs.iter().flat_map(|r| r.pts.iter().map(|p| ((p.0).to_bits(), (p.1 - 1.0).to_bits()))).collect();
        let orig: Vec<_> = a.runs.iter().flat_map(|r| r.pts.iter().map(|p| (p.0.to_bits(), p.1.to_bits()))).collect();
        assert_ne!(shifted, orig, "a moved mouth draws its own tree");
    }

    /// Builds the painted river field for `g` (optionally with `fans`) over the fixture
    /// grid at zero elevation; panics if the field is not built.
    fn field_of(g: &RiverGeometry, fans: Option<&RiverGeometry>) -> crate::river_field::RiverField {
        crate::river_field::build_with(g, fans, GW, GH, 1.0, &|_, _| 0.0).expect("a field")
    }

    /// Protects: the off switch. With no fans the field is bit-identical to the
    /// plain build (`WorldGen::set_river_deltas(false)` takes this path: the
    /// fans are never derived), and so is a fan geometry with no runs; with
    /// fans it differs. And the colour field, likewise.
    #[test]
    fn the_off_switch_is_the_plain_field() {
        let g = RiverGeometry { runs: vec![run(4, 5000.0, 62.5, true)] };
        let plain = crate::river_field::build(&g, GW, GH, 1.0, &|_, _| 0.0).expect("a field");
        let off = field_of(&g, None);
        assert_eq!(plain.bits, off.bits);
        assert_eq!(plain.segments, off.segments);
        let empty = RiverGeometry { runs: vec![] };
        assert_eq!(plain.bits, field_of(&g, Some(&empty)).bits);
        let (fans, _) = delta_fans(&g, GW, GH, 1.0, WORLD_START, &narrow_lake);
        let on = field_of(&g, Some(&fans));
        assert!(on.segments > plain.segments && on.bits != plain.bits, "positive control: the fan changes the field");
    }

    /// Protects: `a_fan_only_adds_coverage` -- at every texel the shader's
    /// coverage with the fans is at least the network's alone (at fit and a
    /// deep density), so no existing river pixel is thinned or removed; the
    /// field changes only within the fans' own reach of the apex and the
    /// shore (no leak elsewhere); and the changed area is large enough to be
    /// a visible fan (a dropped fan changes nothing and fails the count).
    #[test]
    fn a_fan_only_adds_coverage_and_only_near_its_mouth() {
        let g = RiverGeometry { runs: vec![run(4, 5000.0, 62.5, true)] };
        let (fans, st) = delta_fans(&g, GW, GH, 1.0, WORLD_START, &narrow_lake);
        assert!(st.branches >= 2, "{st:?}");
        let base = field_of(&g, None);
        let with = field_of(&g, Some(&fans));
        let setback = 3.0 * SETBACK_PER_ORDER_CELLS;
        let apex = (59.5 - setback, 50.5);
        let reach = MAX_REACH_OVER_SETBACK * setback + 6.0;
        for ppc in [0.5f32, 2.0] {
            let mut gained = 0usize;
            for j in 0..base.h {
                for i in 0..base.w {
                    let (x, y) = ((i as f32 + 0.5) / base.scale as f32, (j as f32 + 0.5) / base.scale as f32);
                    let (cb, _) = crate::river_field::coverage(base.texel(i as i64, j as i64), ppc, 1.0);
                    let (cw, _) = crate::river_field::coverage(with.texel(i as i64, j as i64), ppc, 1.0);
                    assert!(cw + 1e-4 >= cb, "coverage fell at ({x},{y}): {cb} -> {cw}");
                    if cw > cb + 0.05 {
                        gained += 1;
                        let d = ((x - apex.0).powi(2) + (y - apex.1).powi(2)).sqrt();
                        assert!(d <= reach, "coverage gained {d} cells from the apex at ({x},{y}), past the fan's reach {reach}");
                        assert!(x >= apex.0 - 1.0, "a fan texel upstream of its own apex: ({x},{y})");
                    }
                }
            }
            assert!(gained > 30, "ppc {ppc}: the fan painted only {gained} texels more");
        }
    }

    /// Protects: the colour field -- with the fans the colour field covers the
    /// fan's own cells with a river colour (it would sample terrain without
    /// them) and the network's own cells are exactly the plain field's; with
    /// `None` the whole layer is the plain one.
    #[test]
    fn the_colour_field_gets_the_fans_and_keeps_the_network_as_it_was() {
        let a = crate::render::TerrainAppearance::default();
        let g = RiverGeometry { runs: vec![run(4, 5000.0, 62.5, true)] };
        let (fans, _) = delta_fans(&g, GW, GH, 1.0, WORLD_START, &narrow_lake);
        let plain = crate::river_stroke::rasterize_colour_field(&g, &a, GW, GH);
        let none = crate::river_stroke::rasterize_colour_field_with(&g, None, &a, GW, GH);
        let with = crate::river_stroke::rasterize_colour_field_with(&g, Some(&fans), &a, GW, GH);
        let mut fan_cells = 0usize;
        for y in 0..GH {
            for x in 0..GW {
                assert_eq!(plain.at(x, y), none.at(x, y), "None is the plain colour field");
                if (y as i64 - 50).abs() == 0 && x < 60 {
                    assert_eq!(plain.at(x, y), with.at(x, y), "the network's own cells are untouched at ({x},{y})");
                }
                if plain.at(x, y).is_none() && with.at(x, y).is_some() {
                    fan_cells += 1;
                }
            }
        }
        assert!(fan_cells > 100, "the fan reached only {fan_cells} colour cells");
        // A cell on the outer branch's path, well off the river's own band.
        let r = fans.runs.iter().max_by(|p, q| p.pts.last().unwrap().1.partial_cmp(&q.pts.last().unwrap().1).unwrap()).unwrap();
        let mid = r.pts[r.pts.len() / 2];
        assert!(with.at(mid.0 as usize, mid.1 as usize).is_some(), "a branch's own cell has a river colour: {mid:?}");
    }

    /// Protects: RIM-7's one gate. A fan is drawn (screen, tiles, export) only
    /// with the switch on AND a look that paints its rivers (`smooth_shores`
    /// and `rivers_as_water`) AND a grid whose river field fits its texel
    /// budget; each term alone turns it off. The grids are literals, not
    /// `MAX_TEXELS` arithmetic: the owner's 2048x1311 and a 4096x2622 map fit,
    /// an 8192x5240 one does not (`river_field::field_scale`'s `None`) and
    /// keeps the stroke, which draws no fan. Mutating any term goes red.
    #[test]
    fn fans_are_drawn_only_where_the_painted_river_is() {
        let a = crate::render::TerrainAppearance::default();
        assert!(a.smooth_shores && a.rivers_as_water, "precondition: the default look paints its rivers");
        assert!(fans_drawn(&a, false, 2048, 1311), "the owner's world, switch on");
        assert!(fans_drawn(&a, false, 4096, 2622), "a 4096 map fits the field at one texel a cell");
        assert!(!fans_drawn(&a, true, 2048, 1311), "the off switch");
        assert!(!fans_drawn(&a, false, 8192, 5240), "a grid over the field's budget keeps the stroke, which has no fan");
        let stroke_look = crate::render::TerrainAppearance { rivers_as_water: false, ..a.clone() };
        assert!(!fans_drawn(&stroke_look, false, 2048, 1311), "a look that keeps the stroke");
        let cell_coast = crate::render::TerrainAppearance { smooth_shores: false, ..a.clone() };
        assert!(!fans_drawn(&cell_coast, false, 2048, 1311), "a look with no smooth shore paints no river");
        assert!(!fans_drawn(&crate::render::TerrainAppearance::js_reference(), false, 2048, 1311), "the reference look has no river layer");
    }

    /// Protects: the order the tiles and the export stroke in
    /// (`river_stroke::rasterize_with`) -- the fans FIRST, the network over
    /// them, so wherever the network is opaque its pixel is exactly the
    /// network's alone (`rasterize`), whatever colour a fan is; the fan shows
    /// beside the network, and `None` is `rasterize` bit for bit. The fans are
    /// recoloured red here so the order is visible: drawn last, a fan's own
    /// core over the trunk at its apex would turn trunk pixels red, which
    /// fails the first loop.
    #[test]
    fn the_stroke_draws_the_fans_first_and_the_network_over_them() {
        let a = crate::render::TerrainAppearance::default();
        let g = RiverGeometry { runs: vec![run(4, 5000.0, 62.5, true)] };
        let (mut fans, st) = delta_fans(&g, GW, GH, 1.0, WORLD_START, &narrow_lake);
        assert!(st.branches >= 2, "positive control: the order-4 mouth fans, {st:?}");
        for r in &mut fans.runs {
            for c in &mut r.colors {
                *c = [1.0, 0.0, 0.0, 1.0];
            }
        }
        let map = crate::river_stroke::RasterMap { scale: (4.0, 4.0), offset: (0.0, 0.0) };
        let (pw, ph) = (GW * 4, GH * 4);
        let net = crate::river_stroke::rasterize(&g, &a, pw, ph, map);
        let none = crate::river_stroke::rasterize_with(&g, None, &a, pw, ph, map);
        let with = crate::river_stroke::rasterize_with(&g, Some(&fans), &a, pw, ph, map);
        let (mut opaque, mut fan_only) = (0usize, 0usize);
        for y in 0..ph {
            for x in 0..pw {
                assert_eq!(net.at(x, y).map(|p| p.map(f32::to_bits)), none.at(x, y).map(|p| p.map(f32::to_bits)), "None is rasterize at ({x},{y})");
                match net.at(x, y) {
                    Some(p) if p[3] >= 1.0 => {
                        opaque += 1;
                        assert_eq!(with.at(x, y), Some(p), "the network's opaque pixel ({x},{y}) is the network's");
                    }
                    None if with.at(x, y).is_some() => fan_only += 1,
                    _ => {}
                }
            }
        }
        assert!(opaque > 100 && fan_only > 100, "the trunk ({opaque} px) and the fans beside it ({fan_only} px) both drew");
    }

    /// Protects: **no pop at the deep-zoom switch** (RIM-7). The painted path
    /// draws the fan from the river field (`river_field::build_with` and the
    /// shader's [`crate::river_field::coverage`]); the deep-zoom tiles and the
    /// export stroke it (`river_stroke::rasterize_with`). Over the fan's
    /// ground on land (from just
    /// past its apex to the shore -- the tips lie on water, which both paths
    /// hide under the water's own colour), the two cover the same pixels
    /// within the antialiasing band at the densities the painted path is shown
    /// at around the switch (0.5 to 2 px a cell; the owner's world crosses it
    /// near 1.3). A tile path that dropped the fans, or stroked them at another
    /// width, fails the total; one that placed them elsewhere fails the
    /// per-pixel term. The bounds (2 % of the stroke's coverage in total, 3 %
    /// per pixel) are a labelled judgement over the run of 2026-10-04, which
    /// measured 0.16-0.97 % in total and 0.65-2.27 % per pixel at 0.5, 1, 1.5
    /// and 2 (the straight-line case,
    /// `river_field::tests::the_field_covers_what_the_stroke_covers`, holds
    /// 2 %). **Not asserted at 3 or 4 px a cell**, where the same run measured
    /// the field 3.2 % / 4.3 % short of the stroke (5.9 % per pixel): a
    /// branch under half a cell wide is narrowed there by the field's own
    /// quarter-cell bilinear error (`river_field`'s module doc, "Resolution
    /// and memory"), which is the painted path's limit for every thin river,
    /// not the fans'. On the owner's world that density is reached only past
    /// the switch, where the tiles stroked the fan when this was written and,
    /// since RIM-7's second pass, paint it exactly at their own pixels
    /// (`river_field::PaintSource`), with no bilinear error at all.
    #[test]
    fn the_stroked_fan_covers_what_the_painted_fan_covers() {
        let a = crate::render::TerrainAppearance::default();
        let rw = a.river_width as f32;
        let g = RiverGeometry { runs: vec![run(4, 5000.0, 62.5, true)] };
        let (fans, st) = delta_fans(&g, GW, GH, rw, WORLD_START, &narrow_lake);
        assert!(st.branches >= 2, "{st:?}");
        let field = crate::river_field::build_with(&g, Some(&fans), GW, GH, rw, &|_, _| 0.0).expect("a field");
        let plain = crate::river_field::build(&g, GW, GH, rw, &|_, _| 0.0).expect("a field");
        // The apex is at x = 35.5 (59.5 less the order-4 setback of 24);
        // the shore at x = 60.
        let (x_lo, x_hi) = (37.0f32, 59.5f32);
        for ppc in [0.5f32, 1.0, 1.5, 2.0] {
            let (pw, ph) = ((GW as f32 * ppc) as usize, (GH as f32 * ppc) as usize);
            let layer = crate::river_stroke::rasterize_with(&g, Some(&fans), &a, pw, ph, crate::river_stroke::RasterMap { scale: (ppc, ppc), offset: (0.0, 0.0) });
            let (mut stroke_px, mut field_px, mut diff, mut fan_px) = (0.0f64, 0.0f64, 0.0f64, 0.0f64);
            for py in 0..ph {
                for px in 0..pw {
                    let (x, y) = (px as f32 / ppc, py as f32 / ppc);
                    if x < x_lo || x > x_hi {
                        continue;
                    }
                    let s = layer.at(px, py).map_or(0.0, |p| p[3]) as f64;
                    let (c, am) = crate::river_field::coverage(field.sample(x, y), ppc, rw);
                    let fv = (c * am) as f64;
                    let (c0, am0) = crate::river_field::coverage(plain.sample(x, y), ppc, rw);
                    stroke_px += s;
                    field_px += fv;
                    diff += (s - fv).abs();
                    fan_px += (fv - (c0 * am0) as f64).max(0.0);
                }
            }
            // Positive control: the window holds a real fan, not just the trunk.
            assert!(fan_px > 0.3 * field_px, "ppc {ppc}: the fan is {fan_px:.1} of {field_px:.1} covered px");
            let rel = (field_px - stroke_px).abs() / stroke_px;
            assert!(rel < 0.02 && diff / stroke_px < 0.03, "ppc {ppc}: stroke {stroke_px:.1} painted {field_px:.1} ({:.2}%), per-pixel {:.2}%", rel * 100.0, diff / stroke_px * 100.0);
        }
    }
}
