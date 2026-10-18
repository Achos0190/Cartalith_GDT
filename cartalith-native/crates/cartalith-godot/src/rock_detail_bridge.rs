//! **GF-6 §4.10** -- rock-styled deep-zoom detail, wired to the world
//! (`GEOLOGY_FIRST_SCOPE.md` §4.10, §7 GF-6; owner Ruling BH).
//!
//! Its own file with a `#[godot_api(secondary)]` block, like `ef9_bridge.rs`.
//! The styling itself is `render::rock_styled_tile` and its helpers; this file
//! decides **which worlds get it** ([`WorldGen::rock_detail_for_tiles`], read
//! by `lod_snapshot_inputs`) and carries the view switch a probe flips
//! ([`WorldGen::set_rock_lod_detail`]). `lod_cache_key`'s `rd` term reads
//! [`WorldGen::rock_detail_wanted`], so the tiles are re-keyed whenever the
//! answer changes.

use std::sync::Arc;

use godot::prelude::*;

use crate::{render, WorldGen, WorldSource};

impl WorldGen {
    /// Whether the deep-zoom tiles should be styled by the exposed rock:
    /// the view switch is on, and the world's parameters build a rock column
    /// (`geology_model`) whose landform processes read it
    /// (`geology_processes`), read from the same `recompute_params()` the
    /// landmark pass hands `RockView::for_world`. That is the gate GF-6's
    /// knickpoints use, for the same reason: on a world whose erosion was
    /// rock-blind the rock at the surface is only where the surface happens
    /// to cross it, so styling relief by it would claim landforms the
    /// simulation did not make. The app's worlds have the processes off, so
    /// this is `false` for them and their tiles do not move. Says nothing
    /// about whether a column exists; [`Self::rock_detail_for_tiles`] checks.
    pub(crate) fn rock_detail_wanted(&self) -> bool {
        if self.rock_lod_detail_off.get() {
            return false;
        }
        let p = self.recompute_params();
        p.geology_model && p.geology_processes
    }

    /// **GF-6 §4.10**: the exposed rock the deep-zoom tiles are styled by
    /// (`render::build_rock_detail`), or `None`.
    ///
    /// `None` -- never a guessed rock -- unless [`Self::rock_detail_wanted`]
    /// and the world is a **generated** one with a rock column: a loaded save
    /// and an import have none (`Geology::Absent`). Computed per snapshot
    /// (one exposure read per cell); not cached.
    pub(crate) fn rock_detail_for_tiles(&self) -> Option<Arc<render::RockDetail>> {
        if !self.rock_detail_wanted() {
            return None;
        }
        let WorldSource::Generated(ws) = self.source.as_ref()? else { return None };
        let p = self.recompute_params();
        let column = ws.geology.column()?;
        render::build_rock_detail(column, &ws.field, ws.sea_level, p.peak_m, p.gw, p.gh).map(Arc::new)
    }
}

#[godot_api(secondary)]
impl WorldGen {
    /// **GF-6 §4.10's view switch**: whether the deep-zoom tiles are styled
    /// by the exposed rock on a world whose processes read rock. `true` is the
    /// default. `false` draws every tile exactly as it was before GF-6 §4.10,
    /// which is what a probe's positive control compares against. It acts
    /// only on such a world: on the app's worlds (processes off) both states
    /// draw the same tiles. The tiles' cache key carries the answer
    /// (`lod_cache_key`'s `rd`), so the next tile request draws the new
    /// state; the shell must still drop its live tiles
    /// (`invalidate_lod_tiles`) for the change to show. A view setting: never
    /// saved, never a world field.
    #[func]
    fn set_rock_lod_detail(&self, on: bool) {
        self.rock_lod_detail_off.set(!on);
    }

    /// The switch [`Self::set_rock_lod_detail`] sets; `true` by default. Not
    /// whether the current world is styled -- [`Self::rock_lod_detail_active`]
    /// answers that.
    #[func]
    fn rock_lod_detail(&self) -> bool {
        !self.rock_lod_detail_off.get()
    }

    /// **For a probe**: what the styling would act on in the current world,
    /// and where to look at it. `{active}` alone (`false`) when
    /// [`Self::rock_lod_detail_active`] is false; otherwise also `land`
    /// (land cells), `cap_edge`, `ridged` and `terraced` (shares of land: the
    /// cap-edge flag, massive or columnar rock, bedded or fissile rock on a cap
    /// edge -- before the ridge's slope gate, which is per pixel), and the
    /// targets `tests/gf6_rock_ladder.rs` picks, by the same rule, as
    /// `Vector2i` cells where one exists: `granite` (the steepest granite land
    /// cell with no sea within 20 cells) and `cap` (the steepest bedded
    /// cap-edge cell), each 140 cells from every edge on a 4-cell stride.
    /// Picked by inputs (rock, flag, grid slope, distance to the sea), never
    /// by the value under test.
    #[func]
    fn rock_lod_detail_stats(&self) -> VarDictionary {
        let Some(rd) = self.rock_detail_for_tiles() else { return vdict! { "active" => false } };
        let Some(WorldSource::Generated(ws)) = self.source.as_ref() else { return vdict! { "active" => false } };
        let (gw, gh) = (self.gw.max(0) as usize, self.gh.max(0) as usize);
        let f = &ws.field;
        let sea = ws.sea_level as f32;
        let (mut land, mut edge, mut ridged, mut terraced) = (0usize, 0usize, 0usize, 0usize);
        for i in 0..(gw * gh).min(f.len()).min(rd.cells.len()) {
            if f[i] < sea {
                continue;
            }
            land += 1;
            let c = rd.cells[i];
            edge += usize::from(c & render::ROCK_CAP_EDGE_BIT != 0);
            if let Some(st) = render::rock_style(c) {
                ridged += usize::from(st.ridge > 0.0);
                terraced += usize::from(st.terrace > 0.0);
            }
        }
        let share = |k: usize| if land == 0 { 0.0 } else { k as f64 / land as f64 };
        let mut d = vdict! {
            "active" => true,
            "land" => land as i64,
            "cap_edge" => share(edge),
            "ridged" => share(ridged),
            "terraced" => share(terraced),
        };
        let granite = cartalith_terrain::geology::Rock::Granite as u8;
        if let Some(p) = steepest(f, gw, gh, sea, &rd.cells, true, |c| c & !render::ROCK_CAP_EDGE_BIT == granite) {
            d.set("granite", p);
        }
        let bedded = |c: u8| {
            c & render::ROCK_CAP_EDGE_BIT != 0
                && cartalith_terrain::geology::Rock::from_u8(c & !render::ROCK_CAP_EDGE_BIT).is_some_and(|r| r.props().jointing == cartalith_terrain::geology::Jointing::Bedded)
        };
        if let Some(p) = steepest(f, gw, gh, sea, &rd.cells, false, bedded) {
            d.set("cap", p);
        }
        d
    }

    /// Whether the current world's deep-zoom tiles are styled by the exposed
    /// rock: the switch is on, the processes read rock, and the world has a
    /// column. Builds the detail to answer (one exposure read per cell).
    #[func]
    fn rock_lod_detail_active(&self) -> bool {
        self.rock_detail_for_tiles().is_some()
    }
}

/// The steepest land cell (grid central difference, unhalved) whose rock byte
/// passes `keep`, 140 cells from every edge on a 4-cell stride and, with
/// `inland`, no sea within 20 cells (a 5-cell sampled box) --
/// `tests/gf6_rock_ladder.rs`' `pick`, restated because a test file cannot be
/// called from here. `None` when no cell qualifies or the grid is too small.
fn steepest(f: &[f32], gw: usize, gh: usize, sea: f32, cells: &[u8], inland: bool, keep: impl Fn(u8) -> bool) -> Option<Vector2i> {
    let m = 140usize;
    if gw <= 2 * m || gh <= 2 * m || f.len() < gw * gh || cells.len() < gw * gh {
        return None;
    }
    let near_sea = |x: usize, y: usize| -> bool {
        let r = 20i64;
        let mut yy = y as i64 - r;
        while yy <= y as i64 + r {
            let mut xx = x as i64 - r;
            while xx <= x as i64 + r {
                if xx >= 0 && yy >= 0 && (xx as usize) < gw && (yy as usize) < gh && f[yy as usize * gw + xx as usize] < sea {
                    return true;
                }
                xx += 5;
            }
            yy += 5;
        }
        false
    };
    let mut best: Option<(f32, usize, usize)> = None;
    for y in (m..gh - m).step_by(4) {
        for x in (m..gw - m).step_by(4) {
            let i = y * gw + x;
            if f[i] < sea || !keep(cells[i]) || (inland && near_sea(x, y)) {
                continue;
            }
            let s = (f[i + 1] - f[i - 1]).hypot(f[i + gw] - f[i - gw]);
            if best.is_none_or(|b| s > b.0) {
                best = Some((s, x, y));
            }
        }
    }
    best.map(|b| Vector2i::new(b.1 as i32, b.2 as i32))
}
