//! GF-8: lithology painting's engine half (`GEOLOGY_FIRST_SCOPE.md` §4.13,
//! §7 GF-8, bar B11; owner Ruling BJ: *"Hand-painting rock types (like biome
//! painting) is part of BH ... Erosion responds to the painted rock."*).
//!
//! # What it does
//!
//! Two steps, in the order §4.13's commit runs them after its undo push
//! (the undo push is the caller's, because the undo stack lives in
//! `cartalith-godot`: `undo::HeightUndo::push_with_column`; the shell caller,
//! `WorldGen::paint_commit`'s Rock branch, is GF-8's second slice and is not
//! built yet):
//!
//! 1. [`bake`]: the user's three painted layers (surface rock, the rock
//!    beneath, cap thickness) **overwrite** the stored column. There is no
//!    override kept over a generated column: §4.13 bakes, as a sculpt commit
//!    bakes into the heightmap, so there is one column, one save set and one
//!    undo mechanism. Every baked cell is flagged in
//!    [`GeologyColumn::painted`], which is provenance only.
//! 2. [`reerode`]: the rock-reading landform chain is run **twice** over a
//!    padded window `W` around the painted cells, once on the pre-commit
//!    column and once on the baked one, and `E_after − E_before` is added to
//!    the field inside `W`. Re-running erosion on an already-eroded surface
//!    would erode `W` a second time; the difference cancels that to first
//!    order, keeps every earlier sculpt and Erode edit, and leaves the
//!    response to the rock change alone. Where the two runs agree bit for bit
//!    the field and column are not written at all, so painting the rock that
//!    is already there moves nothing (B11's identity, by control flow).
//!
//! # The chain, and what it leaves out
//!
//! The generation's own light pass, in §3.1's order, with the generation's
//! own counts through GF-7's clock: stream power through
//! [`cartalith_erosion::tile::tile_erode_rock`] (the κ slice, its ring pinned)
//! and its isostatic rebound; the threshold hillslope (GF-3); karst (GF-5,
//! behind `geology_karst`); glacial (GF-4, behind `passes.glacial`); coastal
//! (GF-4, behind `passes.coastal`). Each is the same `RockContext` method
//! generation calls, so a switch that is off here was off in generation too.
//!
//! Not re-run, each reported in [`ReerodeReport::not_rerun`] rather than
//! silently dropped: GF-11's construction delta on a constructed world (the
//! structural surface is stored for it, the windowed delta is not built);
//! the carve and the channel network (the shell's recompute re-runs
//! hydrology over `W`'s tiles); and the passes block's evolve cycles,
//! sediment fill, velocity and diffusion, which generation runs after the
//! trace on worlds that turn them on.
//!
//! # What it must never do
//!
//! Touch a cell outside `W` (asserted by B11), move a generated world (it
//! has no caller in `generate_terrain`; it acts only on a user's commit), or
//! read anything but the column and fields it is handed. It refuses rather
//! than guesses: a value outside a layer's legal range refuses the whole bake
//! before any cell is written.

use cartalith_terrain::geology::{m_to_norm, Construction, GeologyColumn, NO_LAYER, ROCK_COUNT};

use crate::{geo_clock, Geology, RockContext, WorldParams, WorldState};

/// The `rock_sub` paint value meaning **"none: single layer"** (§4.13's
/// `rock_sub` layer, "255 = none, single layer"). The same byte as
/// [`NO_LAYER`], which is the column's own and the save format's (§2.6).
pub const SUB_NONE: u8 = NO_LAYER;

/// One `rock_cap` paint class, in metres: value `k` means `k · 10` m, so the
/// layer spans 10 m to 2 550 m (§4.13). **Judgement** from the scope: a
/// flood-basalt pile can reach about 2 km, which fits, and 10 m is well under
/// one cell's relief at the default extent.
pub const CAP_CLASS_M: f64 = 10.0;

/// `P`, the padding in cells added on each side of the painted cells'
/// bounding box to make the re-erosion window `W` (§4.13 step 3).
///
/// **Judgement**, from what can carry a rock change sideways in the chain:
/// the threshold hillslope moves talus one cell per pass and runs
/// `THRESHOLD_HILLSLOPE_PASSES` = 8 at τ = 1; the rebound is a Gaussian of
/// `tect.blur_r`-scaled width; stream power's implicit update reaches up a
/// receiver chain but stops at the window's pinned ring. 16 cells is twice
/// the hillslope reach. The ring step this leaves is measured by
/// `tests/gf8_rock_paint.rs` (`b11_shale_into_granite_*`) against §5's B11
/// bar (under 2 % of `W`'s relief) and printed on every run.
pub const REERODE_PAD_CELLS: usize = 16;

/// The user's committed rock paint, one byte per world cell per layer;
/// `0` is "not painted" on every layer (`cartalith_spatial::paint`'s
/// convention).
/// - `top`: rock index + 1 (`1 ..= 11`).
/// - `sub`: rock index + 1, or [`SUB_NONE`] for "single layer".
/// - `cap`: cap thickness class `k` (`1 ..= 255`, `k · CAP_CLASS_M` m).
#[derive(Clone, Copy)]
pub struct RockPaint<'a> {
    pub top: &'a [u8],
    pub sub: &'a [u8],
    pub cap: &'a [u8],
}

/// Why a rock-paint commit was refused. Each is a reason, never a value, and
/// a refused commit writes nothing.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RockPaintRefusal {
    /// The world has no geology column (§2.6's reader rule: the Rock target
    /// is disabled with the column's absence reason).
    NoColumn(&'static str),
    /// A layer is not one byte per world cell.
    Length { layer: &'static str, len: usize, need: usize },
    /// A layer holds a value outside its legal range (see [`RockPaint`]).
    BadValue { layer: &'static str, cell: usize, value: u8 },
}

impl RockPaintRefusal {
    /// The user-facing sentence.
    pub fn reason(&self) -> String {
        match self {
            RockPaintRefusal::NoColumn(why) => format!("no geology column: {why}"),
            RockPaintRefusal::Length { layer, len, need } => {
                format!("the {layer} layer is {len} cells and the world is {need}")
            }
            RockPaintRefusal::BadValue { layer, cell, value } => {
                format!("the {layer} layer holds {value} at cell {cell}, which is not a legal value")
            }
        }
    }
}

/// What [`bake`] did.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BakeReport {
    /// Cells where at least one layer's paint was applied (each now has
    /// `painted = 1`).
    pub cells_baked: usize,
    /// Cells whose `rock_top`, `rock_sub` or `contact` bytes actually
    /// changed. Smaller than `cells_baked` when a dab repaints what was there.
    pub cells_changed: usize,
    /// Cap-thickness dabs skipped because the cell has no rock beneath
    /// (§4.13: "a cap needs something beneath it, and inventing a substrate
    /// would be a silent choice").
    pub cap_skipped: usize,
    /// Substrate dabs skipped on a single-layer cell that was given no cap
    /// thickness in the same commit: the contact would have no elevation.
    /// The same rule as `cap_skipped`, from the other side; a **labelled
    /// judgement** (the scope is silent on this case), stated so it is never
    /// silent.
    pub sub_skipped: usize,
    /// Inclusive bounding box `(x0, y0, x1, y1)` of the baked cells; `None`
    /// when nothing was baked.
    pub bbox: Option<(usize, usize, usize, usize)>,
}

/// Bakes `paint` into `column` (§4.13 step 2). `field` is the surface at
/// commit time; `sea` and `peak_m` convert a cap thickness to normalised
/// units (`m_to_norm`).
///
/// Per cell, in this order: the substrate (`SUB_NONE` makes the cell single
/// layer and its contact NaN, §2.6); then the cap thickness `t`, which sets
/// `contact = (field − regolith) − t` in normalised units, so the contact is
/// absolute (§2.5) and later lowering breaches it; then the surface rock.
///
/// Every value is validated before any cell is written, so a refusal leaves
/// `column` untouched. Never invents a substrate or a contact.
pub fn bake(
    column: &mut GeologyColumn,
    field: &[f32],
    paint: &RockPaint,
    gw: usize,
    sea: f64,
    peak_m: f64,
) -> Result<BakeReport, RockPaintRefusal> {
    let n = column.len();
    for (layer, len) in [("field", field.len()), ("rock_top", paint.top.len()), ("rock_sub", paint.sub.len()), ("rock_cap", paint.cap.len())] {
        if len != n {
            return Err(RockPaintRefusal::Length { layer, len, need: n });
        }
    }
    let top_ok = |v: u8| v == 0 || (v as usize) <= ROCK_COUNT;
    let sub_ok = |v: u8| v == 0 || v == SUB_NONE || (v as usize) <= ROCK_COUNT;
    for (cell, (&t, &s)) in paint.top.iter().zip(paint.sub.iter()).enumerate() {
        if !top_ok(t) {
            return Err(RockPaintRefusal::BadValue { layer: "rock_top", cell, value: t });
        }
        if !sub_ok(s) {
            return Err(RockPaintRefusal::BadValue { layer: "rock_sub", cell, value: s });
        }
    }
    // `rock_cap` is legal at every byte (0 = unpainted, 1..=255 a class).

    let mut rep = BakeReport::default();
    let mut painted = column.painted.take().unwrap_or_else(|| vec![0u8; n]);
    let mut any = false;
    for i in 0..n {
        let (t, s, c) = (paint.top[i], paint.sub[i], paint.cap[i]);
        if t == 0 && s == 0 && c == 0 {
            continue;
        }
        let (old_top, old_sub, old_contact) = (column.rock_top[i], column.rock_sub[i], column.contact[i]);
        let mut sub = old_sub;
        let mut contact = old_contact;
        let mut applied = false;
        if s == SUB_NONE {
            sub = NO_LAYER;
            contact = f32::NAN;
            applied = true;
        } else if s != 0 {
            if sub == NO_LAYER && c == 0 {
                rep.sub_skipped += 1;
            } else {
                sub = s - 1;
                applied = true;
            }
        }
        if c != 0 {
            if sub == NO_LAYER {
                rep.cap_skipped += 1;
            } else {
                let bedrock = field[i] as f64 - column.regolith[i] as f64;
                contact = (bedrock - m_to_norm(c as f64 * CAP_CLASS_M, sea, peak_m)) as f32;
                applied = true;
            }
        }
        let top = if t != 0 {
            applied = true;
            t - 1
        } else {
            old_top
        };
        if !applied {
            continue;
        }
        if top != old_top || sub != old_sub || contact.to_bits() != old_contact.to_bits() {
            rep.cells_changed += 1;
        }
        column.rock_top[i] = top;
        column.rock_sub[i] = sub;
        column.contact[i] = contact;
        painted[i] = 1;
        any = true;
        rep.cells_baked += 1;
        let (x, y) = (i % gw, i / gw);
        rep.bbox = Some(match rep.bbox {
            None => (x, y, x, y),
            Some((x0, y0, x1, y1)) => (x0.min(x), y0.min(y), x1.max(x), y1.max(y)),
        });
    }
    // Absent stays absent: a commit that baked nothing on a never-painted
    // world must not create an all-zero mask (§2.6).
    column.painted = if any || painted.iter().any(|&v| v != 0) { Some(painted) } else { None };
    Ok(rep)
}

/// A rectangle of world cells, `x0 .. x0 + w` by `y0 .. y0 + h`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Window {
    pub x0: usize,
    pub y0: usize,
    pub w: usize,
    pub h: usize,
}

impl Window {
    /// The inclusive box `bbox` padded by `pad` cells on each side and
    /// clipped to the `gw × gh` map. A wrapping world is clipped at its seam
    /// too: the window does not wrap (the tile kernel refuses to).
    pub fn padded(bbox: (usize, usize, usize, usize), pad: usize, gw: usize, gh: usize) -> Window {
        let (x0, y0, x1, y1) = bbox;
        let (ax, ay) = (x0.saturating_sub(pad), y0.saturating_sub(pad));
        let (bx, by) = ((x1 + pad).min(gw - 1), (y1 + pad).min(gh - 1));
        Window { x0: ax, y0: ay, w: bx - ax + 1, h: by - ay + 1 }
    }

    /// The world index of window cell `(x, y)`.
    fn world(&self, x: usize, y: usize, gw: usize) -> usize {
        (self.y0 + y) * gw + self.x0 + x
    }
}

/// Why [`reerode`] did not run. The bake still stands; the field is untouched.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReerodeSkip {
    /// `geology_processes` is off: no landform process reads rock on this
    /// world, so the differential is zero by definition and is not computed.
    ProcessesOff,
    /// `carve_rivers` is off: generation ran no light pass, so there is no
    /// generation chain to repeat.
    CarveRiversOff,
    /// The window is under 3 cells on an axis: it is all pinned ring.
    WindowTooSmall,
}

impl ReerodeSkip {
    /// The user-facing sentence.
    pub fn reason(self) -> &'static str {
        match self {
            ReerodeSkip::ProcessesOff => "geology processes are off: erosion does not read rock on this world",
            ReerodeSkip::CarveRiversOff => "river carving is off: generation ran no light erosion pass to repeat",
            ReerodeSkip::WindowTooSmall => "the painted window is under 3 cells on an axis",
        }
    }
}

/// What [`reerode`] did.
#[derive(Clone, Debug, PartialEq)]
pub struct ReerodeReport {
    pub window: Window,
    /// The chain stages that ran, in order.
    pub stages: Vec<&'static str>,
    /// Stages generation may have run that this chain does not repeat, each
    /// with its reason.
    pub not_rerun: Vec<&'static str>,
    /// Cells whose height moved (bit pattern changed).
    pub cells_moved: usize,
    /// Largest |Δ| on a cell of `W`'s edge that borders an unchanged cell
    /// outside `W` (map-edge sides excluded): the step the window opens at its
    /// ring, normalised units.
    pub ring_step: f64,
    /// `W`'s relief before the commit (max − min of the field), normalised.
    pub relief: f64,
}

/// Copies window `win` of a world-sized slice.
fn cut<T: Copy>(v: &[T], win: &Window, gw: usize) -> Vec<T> {
    let mut out = Vec::with_capacity(win.w * win.h);
    for y in 0..win.h {
        let s = win.world(0, y, gw);
        out.extend_from_slice(&v[s..s + win.w]);
    }
    out
}

/// Window `win` of a column. `construction` is carried as `Off` because no
/// chain stage reads it (the constructed-world decision is taken once, on
/// the world column, in [`reerode`]); `painted` is not carried (provenance,
/// read by nothing in the chain).
fn cut_column(c: &GeologyColumn, win: &Window, gw: usize) -> GeologyColumn {
    GeologyColumn {
        rock_top: cut(&c.rock_top, win, gw),
        rock_sub: cut(&c.rock_sub, win, gw),
        contact: cut(&c.contact, win, gw),
        regolith: cut(&c.regolith, win, gw),
        volcanic_setting: cut(&c.volcanic_setting, win, gw),
        construction: Construction::Off,
        painted: None,
    }
}

/// The world fields one chain run reads, cut to the window.
struct WindowInputs {
    stress: Vec<f32>,
    rain: Vec<f32>,
    temperature: Vec<f32>,
    resistance: Vec<f32>,
}

/// One run of the chain over a window (see the module doc for the list).
/// Returns the column the run leaves; `field` is eroded in place.
#[allow(clippy::too_many_arguments)]
fn run_chain(
    rock: &RockContext,
    p: &WorldParams,
    field: &mut Vec<f32>,
    column: GeologyColumn,
    inp: &WindowInputs,
    sw: usize,
    sh: usize,
    gw: usize,
    sea: f64,
    constructed: bool,
    stages: &mut Vec<&'static str>,
) -> GeologyColumn {
    let mut geo = Geology::Column(Box::new(column));
    // Generation's light pass: skipped on a constructed world, and by the
    // streak spike's ablation, exactly as `generate_terrain_inner` skips it.
    if !constructed && !p.streak.skip_light_pass {
        let pre = field.clone();
        let sp = cartalith_erosion::StreamPowerParams {
            k: p.stream.k,
            uplift: p.stream.uplift,
            deposit: p.stream.deposit,
            climate_k: p.stream.climate_k,
            iters: rock.clock.light_pass_iters(p.stream.iters).n,
            resist: p.tect.resist,
            g: p.planet.g,
            world: false,
            sea,
        };
        let column = geo.column_mut().expect("run_chain is built on a column");
        // `refine = 1`: a same-resolution re-run, so the area seed is the
        // world's own 1.0 per cell (`uniform_area_seed`'s degenerate case).
        cartalith_erosion::tile::tile_erode_rock(
            field,
            &inp.stress,
            &inp.rain,
            sw,
            sh,
            &sp,
            1,
            &cartalith_erosion::tile::uniform_area_seed(sw, sh, 1),
            &mut cartalith_erosion::StreamPowerRock { column: &mut *column, contrast: rock.contrast, r_expose: rock.r_expose },
        );
        cartalith_erosion::account_regolith(column, &pre, field, true);
        rock.rebound(&mut geo, field, &pre, sw, sh, p.tect.blur_r, false);
        stages.push("stream power (tile, κ slice) + rebound");
    }
    // `cell_m` must stay the world's: the window is `sw` of the world's `gw`
    // cells, so its width in km is scaled by the same ratio.
    let win_km = p.map_width_km * sw as f64 / gw as f64;
    rock.threshold_hillslope(&mut geo, field, sw, sh, false, win_km, sea, p.peak_m);
    stages.push("threshold hillslope");
    if p.geology_karst {
        rock.karst(true, &mut geo, field, &inp.rain, sea, p.peak_m);
        stages.push("karst");
    }
    if p.passes.glacial {
        // `resistance` is a scratch copy: `dynamic_lithology`'s update is not
        // written back (no rock kernel reads `resistance_field`).
        let mut resistance = inp.resistance.clone();
        rock.glacial(&mut geo, &mut resistance, field, &inp.temperature, sw, sh, sea, false, p);
        stages.push("glacial");
    }
    if p.passes.coastal {
        let flow = cartalith_hydrology::compute_flow_routed(sw, sh, field, Some(&inp.rain), true, false, sea, p.integrate_drainage);
        rock.coastal(
            &mut geo,
            field,
            &flow,
            sw,
            sh,
            sea,
            false,
            p.planet.g,
            &cartalith_erosion::CoastalParams {
                wave_str: p.passes.wave_str,
                estuary_depth: p.passes.estuary_depth,
                marsh_band: p.passes.marsh_band,
                passes: rock.clock.coastal_passes(p.passes.coastal_passes).n,
            },
        );
        stages.push("coastal");
    }
    match geo {
        Geology::Column(c) => *c,
        Geology::Absent(_) => unreachable!("run_chain never removes the column"),
    }
}

/// `(x − y)` in `f64` for two `f32`s, `0` when either is NaN: the change a
/// run made to a contact, with a single-layer cell (NaN) changing by nothing.
fn nan0_delta(out: f32, inp: f32) -> f64 {
    if out.is_nan() || inp.is_nan() {
        0.0
    } else {
        out as f64 - inp as f64
    }
}

/// §4.13 step 3: the differential re-erosion of window `win`.
///
/// `before` is the column as it was before [`bake`] ran; `ws.geology` holds
/// the baked one. `p` is the world's parameters (the counts and switches the
/// chain repeats). Writes `ws.field` and the baked column's `regolith` and
/// `contact` inside `win`, and only where the two runs disagree.
///
/// `gw × gh` is the world grid (`WorldState` does not carry it; the caller's
/// `WorldParams` or `WorldGen` does).
pub fn reerode(
    ws: &mut WorldState,
    p: &WorldParams,
    gw: usize,
    gh: usize,
    before: &GeologyColumn,
    win: Window,
) -> Result<ReerodeReport, ReerodeSkip> {
    if !p.geology_processes {
        return Err(ReerodeSkip::ProcessesOff);
    }
    if !p.carve_rivers {
        return Err(ReerodeSkip::CarveRiversOff);
    }
    if win.w < 3 || win.h < 3 {
        return Err(ReerodeSkip::WindowTooSmall);
    }
    let sea = ws.sea_level;
    let Some(after) = ws.geology.column() else {
        // The caller baked into this column a line ago; reaching here
        // without one is the same gate as "processes off".
        return Err(ReerodeSkip::ProcessesOff);
    };
    let rock = RockContext {
        on: true,
        contrast: p.tect.resist,
        r_expose: m_to_norm(cartalith_terrain::geology::R_EXPOSE_M, sea, p.peak_m) as f32,
        clock: geo_clock::GeoClock::new(true, p.geo_age),
    };
    let constructed = matches!(after.construction, Construction::Built { .. });
    let mut not_rerun = vec![
        "carve and channel network (re-run by the recompute over W's tiles)",
    ];
    if constructed {
        not_rerun.push("construction delta (GF-11): the windowed delta is not built");
    }
    if p.passes.evolve_cycles > 0 || p.passes.sediment_fill || p.passes.velocity || p.passes.hillslope {
        not_rerun.push("passes block (evolve cycles, sediment fill, velocity, diffusion)");
    }

    let (sw, sh) = (win.w, win.h);
    let field_w = cut(&ws.field, &win, gw);
    let inp = WindowInputs {
        stress: cut(&ws.stress_field, &win, gw),
        rain: cut(&ws.rainfall, &win, gw),
        temperature: cut(&ws.temperature, &win, gw),
        resistance: cut(&ws.resistance_field, &win, gw),
    };
    let col_b_in = cut_column(before, &win, gw);
    let col_a_in = cut_column(after, &win, gw);

    let mut stages = Vec::new();
    let mut f_b = field_w.clone();
    let col_b = run_chain(&rock, p, &mut f_b, col_b_in.clone(), &inp, sw, sh, gw, sea, constructed, &mut Vec::new());
    let mut f_a = field_w.clone();
    let col_a = run_chain(&rock, p, &mut f_a, col_a_in.clone(), &inp, sw, sh, gw, sea, constructed, &mut stages);

    let relief = {
        let (mn, mx) = field_w.iter().fold((f32::INFINITY, f32::NEG_INFINITY), |(a, b), &v| (a.min(v), b.max(v)));
        (mx - mn) as f64
    };
    let field = std::sync::Arc::make_mut(&mut ws.field);
    let column = ws.geology.column_mut().expect("checked above");
    let mut cells_moved = 0usize;
    let mut ring_step = 0.0f64;
    for y in 0..sh {
        for x in 0..sw {
            let k = y * sw + x;
            let i = win.world(x, y, gw);
            if f_a[k].to_bits() != f_b[k].to_bits() {
                // `field_in + (a − in) − (b − in)`, which is `field_in + a − b`.
                let v = (field[i] as f64 + f_a[k] as f64 - f_b[k] as f64).clamp(0.0, 1.0) as f32;
                if v.to_bits() != field[i].to_bits() {
                    cells_moved += 1;
                    let d = (v as f64 - field[i] as f64).abs();
                    let borders_outside = (x == 0 && win.x0 > 0)
                        || (y == 0 && win.y0 > 0)
                        || (x + 1 == sw && win.x0 + sw < gw)
                        || (y + 1 == sh && win.y0 + sh < gh);
                    if borders_outside {
                        ring_step = ring_step.max(d);
                    }
                    field[i] = v;
                }
            }
            if col_a.regolith[k].to_bits() != col_b.regolith[k].to_bits() || col_a_in.regolith[k].to_bits() != col_b_in.regolith[k].to_bits() {
                let d = (col_a.regolith[k] as f64 - col_a_in.regolith[k] as f64) - (col_b.regolith[k] as f64 - col_b_in.regolith[k] as f64);
                if d != 0.0 {
                    column.regolith[i] = (column.regolith[i] as f64 + d).max(0.0) as f32;
                }
            }
            if !column.contact[i].is_nan() {
                let d = nan0_delta(col_a.contact[k], col_a_in.contact[k]) - nan0_delta(col_b.contact[k], col_b_in.contact[k]);
                if d != 0.0 {
                    column.contact[i] = (column.contact[i] as f64 + d) as f32;
                }
            }
        }
    }
    Ok(ReerodeReport { window: win, stages, not_rerun, cells_moved, ring_step, relief })
}

/// What [`commit`] did: the bake, and the re-erosion or why it did not run.
#[derive(Clone, Debug)]
pub struct CommitReport {
    pub bake: BakeReport,
    pub reerode: Result<ReerodeReport, ReerodeSkip>,
}

/// §4.13's commit after the undo push: [`bake`], then [`reerode`] over the
/// baked cells' bounding box padded by `pad` cells ([`REERODE_PAD_CELLS`] in
/// the app). A bake that changed no rock byte still re-runs nothing harmful:
/// both chain runs agree and nothing is written.
pub fn commit(
    ws: &mut WorldState,
    p: &WorldParams,
    gw: usize,
    gh: usize,
    paint: &RockPaint,
    pad: usize,
) -> Result<CommitReport, RockPaintRefusal> {
    let sea = ws.sea_level;
    let field = std::sync::Arc::clone(&ws.field);
    let column = match &mut ws.geology {
        Geology::Column(c) => c,
        Geology::Absent(why) => return Err(RockPaintRefusal::NoColumn(why.reason())),
    };
    let before = (**column).clone();
    let bake_rep = bake(column, &field, paint, gw, sea, p.peak_m)?;
    drop(field);
    let reerode = match bake_rep.bbox {
        None => Err(ReerodeSkip::WindowTooSmall),
        Some(bbox) => reerode(ws, p, gw, gh, &before, Window::padded(bbox, pad, gw, gh)),
    };
    Ok(CommitReport { bake: bake_rep, reerode })
}

/// Unit tests for [`bake`] and [`Window::padded`] on literal fixtures: the
/// per-cell rules of §4.13 step 2, the refusal-writes-nothing rule and the
/// absent-stays-absent rule. The re-erosion is tested on generated worlds in
/// `tests/gf8_rock_paint.rs`.
#[cfg(test)]
mod tests {
    use super::*;
    use cartalith_terrain::geology::Rock;

    /// A 4-cell, 4-wide column: cell 0 single-layer granite, cell 1 granite
    /// over shale with contact 0.5, cells 2-3 single-layer limestone.
    fn col() -> GeologyColumn {
        GeologyColumn {
            rock_top: vec![Rock::Granite as u8, Rock::Granite as u8, Rock::Limestone as u8, Rock::Limestone as u8],
            rock_sub: vec![NO_LAYER, Rock::Shale as u8, NO_LAYER, NO_LAYER],
            contact: vec![f32::NAN, 0.5, f32::NAN, f32::NAN],
            regolith: vec![0.0, 0.0, 0.01, 0.0],
            volcanic_setting: vec![0; 4],
            construction: Construction::Off,
            painted: None,
        }
    }

    /// Bit view, for exact equality including NaN.
    fn bits(v: &[f32]) -> Vec<u32> {
        v.iter().map(|x| x.to_bits()).collect()
    }

    #[test]
    fn surface_paint_overwrites_the_top_and_flags_the_cell() {
        // Protects: step 2's first rule and the provenance mask.
        let mut c = col();
        let f = vec![0.6f32; 4];
        let r = bake(&mut c, &f, &RockPaint { top: &[10, 0, 0, 0], sub: &[0; 4], cap: &[0; 4] }, 4, 0.3, 3000.0).unwrap();
        assert_eq!(c.rock_top[0], Rock::Shale as u8);
        assert_eq!(c.painted.as_deref(), Some(&[1u8, 0, 0, 0][..]));
        assert_eq!((r.cells_baked, r.cells_changed, r.bbox), (1, 1, Some((0, 0, 0, 0))));
    }

    #[test]
    fn repainting_the_same_rock_changes_no_rock_byte() {
        // Protects: B11's column half -- only `painted` moves.
        let mut c = col();
        let before = c.clone();
        let r = bake(&mut c, &[0.6; 4], &RockPaint { top: &[1, 1, 0, 0], sub: &[0; 4], cap: &[0; 4] }, 4, 0.3, 3000.0).unwrap();
        assert_eq!((r.cells_baked, r.cells_changed), (2, 0));
        assert_eq!(c.rock_top, before.rock_top);
        assert_eq!(c.rock_sub, before.rock_sub);
        assert_eq!(bits(&c.contact), bits(&before.contact));
        assert_eq!(c.painted.as_deref(), Some(&[1u8, 1, 0, 0][..]));
    }

    #[test]
    fn cap_thickness_sets_an_absolute_contact_below_the_bedrock_surface() {
        // Protects: `contact = (field − regolith) − t·(1 − sea)/peak_m`, with
        // literal numbers: field 0.6, regolith 0, t = 3·10 m, sea 0.3,
        // peak 3000 m → 0.6 − 30·0.7/3000 = 0.593.
        let mut c = col();
        bake(&mut c, &[0.6; 4], &RockPaint { top: &[0; 4], sub: &[0; 4], cap: &[0, 3, 0, 0] }, 4, 0.3, 3000.0).unwrap();
        assert!((c.contact[1] - 0.593).abs() < 1e-6, "contact {}", c.contact[1]);
        // With regolith 0.01 under it the bedrock surface is 0.59.
        let mut c = col();
        bake(&mut c, &[0.6; 4], &RockPaint { top: &[0; 4], sub: &[0, 0, 10, 0], cap: &[0, 0, 3, 0] }, 4, 0.3, 3000.0).unwrap();
        assert_eq!(c.rock_sub[2], Rock::Shale as u8);
        assert!((c.contact[2] - 0.583).abs() < 1e-6, "contact {}", c.contact[2]);
    }

    #[test]
    fn a_cap_on_a_single_layer_cell_is_skipped_and_counted() {
        // Protects: "a cap needs something beneath it" -- nothing invented.
        let mut c = col();
        let r = bake(&mut c, &[0.6; 4], &RockPaint { top: &[0; 4], sub: &[0; 4], cap: &[5, 0, 0, 0] }, 4, 0.3, 3000.0).unwrap();
        assert_eq!((r.cap_skipped, r.cells_baked), (1, 0));
        assert!(c.contact[0].is_nan());
        assert!(c.painted.is_none(), "nothing baked: the mask stays absent");
    }

    #[test]
    fn a_substrate_without_a_cap_on_a_single_layer_cell_is_skipped() {
        // Protects: the labelled judgement -- no contact is invented.
        let mut c = col();
        let r = bake(&mut c, &[0.6; 4], &RockPaint { top: &[0; 4], sub: &[10, 0, 0, 0], cap: &[0; 4] }, 4, 0.3, 3000.0).unwrap();
        assert_eq!((r.sub_skipped, r.cells_baked), (1, 0));
        assert_eq!(c.rock_sub[0], NO_LAYER);
    }

    #[test]
    fn substrate_none_makes_the_cell_single_layer_with_a_nan_contact() {
        // Protects: §2.6's NaN-where-no-second-layer rule.
        let mut c = col();
        bake(&mut c, &[0.6; 4], &RockPaint { top: &[0; 4], sub: &[0, SUB_NONE, 0, 0], cap: &[0; 4] }, 4, 0.3, 3000.0).unwrap();
        assert_eq!(c.rock_sub[1], NO_LAYER);
        assert!(c.contact[1].is_nan());
        assert!(c.substrate(1).is_none());
    }

    #[test]
    fn an_illegal_value_refuses_and_writes_nothing() {
        // Protects: validation before any write. 12 is one past the 11
        // rocks' `index + 1` range; 200 is neither a rock nor SUB_NONE.
        let mut c = col();
        let e = bake(&mut c, &[0.6; 4], &RockPaint { top: &[1, 12, 0, 0], sub: &[0; 4], cap: &[0; 4] }, 4, 0.3, 3000.0).unwrap_err();
        assert_eq!(e, RockPaintRefusal::BadValue { layer: "rock_top", cell: 1, value: 12 });
        assert_eq!(c.rock_top[0], Rock::Granite as u8, "cell 0 was valid but must not be written");
        let e = bake(&mut c, &[0.6; 4], &RockPaint { top: &[0; 4], sub: &[0, 0, 200, 0], cap: &[0; 4] }, 4, 0.3, 3000.0).unwrap_err();
        assert_eq!(e, RockPaintRefusal::BadValue { layer: "rock_sub", cell: 2, value: 200 });
        assert!(c.painted.is_none());
        // 11 (Unconsolidated + 1) is the top of the legal range.
        assert!(bake(&mut c, &[0.6; 4], &RockPaint { top: &[11, 0, 0, 0], sub: &[0; 4], cap: &[0; 4] }, 4, 0.3, 3000.0).is_ok());
    }

    #[test]
    fn the_window_is_padded_and_clipped() {
        // Protects: `W` = bbox ± P, clipped to the map.
        assert_eq!(Window::padded((10, 10, 12, 11), 4, 100, 50), Window { x0: 6, y0: 6, w: 11, h: 10 });
        assert_eq!(Window::padded((1, 48, 99, 49), 4, 100, 50), Window { x0: 0, y0: 44, w: 100, h: 6 });
    }
}
