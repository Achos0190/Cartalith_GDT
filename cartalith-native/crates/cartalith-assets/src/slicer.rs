//! The sprite-sheet slicer's arithmetic and pixel work — the reference's
//! `SpriteSheetImporter` (`Cartalith Gen1 v2.10.html`, lines 27464-27870),
//! minus its canvas, its pointer handling and its DOM.
//!
//! # What the reference actually does (read, not assumed)
//!
//! `DCC_SHELL_SPEC.md` §8 describes the slicer modal as *Columns / Rows /
//! Margin / Spacing*, *Trim transparent edges*, *Skip empty cells*. The
//! reference's own control set is close but **not** the same, and the
//! difference is worth stating once here rather than discovering it twice:
//!
//! | §8's control | The reference's reality |
//! |---|---|
//! | Columns / Rows | `#alSlCols` / `#alSlRows`, clamped 1..=128 ([`clamp_grid_count`]) |
//! | Spacing | `#alSlSpacing`, and it is a **half-gutter on interior edges only** — see [`compute_cells`] |
//! | Margin | no such control: the reference has a *draggable* `gridRect` instead ([`GridRect`]), of which a uniform margin is one case |
//! | Skip empty cells | `#alSlSkip`, real — `isBlank`, alpha **> 8** ([`is_blank`]) |
//! | Trim transparent edges | **does not exist.** The reference's second pixel toggle is *background → transparent*, a chroma key ([`apply_chroma`]) |
//!
//! So [`apply_chroma`] and [`is_blank`] are ports. [`trim_transparent_edges`]
//! is not — it is a deliberate port-side addition, named in
//! `DCC_SHELL_SPEC.md` §8 and `GUI_GAP_REGISTER.md` AS-10, built out of the
//! reference's *own* alpha threshold ([`BLANK_ALPHA_THRESHOLD`]) so it agrees
//! with `isBlank` about what "empty" means rather than inventing a second
//! notion. It is disclosed in `cartalith-native/docs/CHANGELOG.md`, per
//! `CLAUDE.md`'s "do not deviate silently" rule, and nothing that *is* a port
//! depends on it.
//!
//! # Golden-verified
//!
//! `tests/golden_parity_slicer.rs` carries a Node `vm` extraction of
//! `computeCells`, `cropCell`'s source-rect rounding, `isBlank` and
//! `applyChroma`, lifted straight out of the frozen HTML by line range
//! (27465-27870, the whole object literal — it executes no DOM at definition
//! time, which is what makes the lift possible at all). The pixel *copy*
//! inside `cropCell` is `ctx.drawImage`, a DOM API with no headless
//! equivalent, so what is golden-verified there is its rounding and clipping
//! geometry, with the blit itself covered by real unit tests — the same
//! carve-out [`crate::raster`]'s module docs already draw for `renderItem`
//! and `itemHash`.

use crate::raster::DecodedImage;
use cartalith_jsmath::{js_min, js_round};

/// `isBlank`'s own alpha threshold (reference line 27770): a pixel counts as
/// content when its alpha is **strictly greater than 8**, so an all-alpha-8
/// cell is still "blank". Ported verbatim; [`trim_transparent_edges`] reuses
/// it deliberately rather than picking a second one.
pub const BLANK_ALPHA_THRESHOLD: u8 = 8;

/// `clampInt`'s bounds for `#alSlCols`/`#alSlRows` (reference line 27591).
pub const MAX_GRID_COUNT: u32 = 128;

// ---------------------------------------------------------------------------
// Grid geometry
// ---------------------------------------------------------------------------

/// The reference's `gridRect` — the sub-rectangle of the sheet the grid
/// covers, in sheet pixels. The reference lets the user drag it; this port
/// exposes the two constructions a UI needs: [`GridRect::whole`] (the
/// reference's own default and its `Reset grid` button) and
/// [`GridRect::inset`] (§8's *Margin* control, which is a uniform inset of
/// exactly this rectangle).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GridRect {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

impl GridRect {
    /// The whole sheet — `loadSheet`'s `this.gridRect={x:0,y:0,w,h}`.
    #[must_use]
    pub fn whole(sheet_w: u32, sheet_h: u32) -> Self {
        GridRect { x: 0.0, y: 0.0, w: f64::from(sheet_w), h: f64::from(sheet_h) }
    }

    /// The whole sheet inset by `margin` on all four sides — §8's *Margin*
    /// control expressed in the reference's own `gridRect` terms. `None`
    /// when the margin leaves nothing behind, so a caller reports that
    /// rather than slicing a negative rectangle.
    #[must_use]
    pub fn inset(sheet_w: u32, sheet_h: u32, margin: f64) -> Option<Self> {
        let m = if margin.is_finite() { margin.max(0.0) } else { 0.0 };
        let w = f64::from(sheet_w) - m * 2.0;
        let h = f64::from(sheet_h) - m * 2.0;
        (w > 0.0 && h > 0.0).then_some(GridRect { x: m, y: m, w, h })
    }
}

/// One cell of the computed grid, in sheet pixels — the reference's
/// `{c,r,idx,x,y,w,h}` (line 27598). `w`/`h` can go negative when spacing
/// exceeds the cell pitch; the reference produces those too, and reports the
/// condition rather than clamping ("Grid too dense", line 27696).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CellRect {
    pub col: u32,
    pub row: u32,
    pub index: usize,
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

/// `computeCells`'s return value: the cells plus the smallest cell width and
/// height across all of them (the reference's `cw`/`ch`, which drive its
/// "min ~W×Hpx" readout and its too-dense guard).
#[derive(Debug, Clone, PartialEq)]
pub struct CellGrid {
    pub cells: Vec<CellRect>,
    pub cols: u32,
    pub rows: u32,
    pub min_w: f64,
    pub min_h: f64,
}

impl CellGrid {
    /// The reference's own `cw>0&&ch>0` test (line 27676/27695): below this,
    /// it refuses to draw the overlay and says "Grid too dense — reduce
    /// columns/rows or spacing."
    #[must_use]
    pub fn is_usable(&self) -> bool {
        self.min_w > 0.0 && self.min_h > 0.0
    }

    /// Each column's `(left, right)` in sheet pixels — the first row's cells,
    /// which is all a grid *overlay* needs to draw the same lines the slice
    /// will cut on. Exposed so a presentation layer never has to reimplement
    /// [`compute_cells`]'s half-gutter arithmetic and drift from it.
    #[must_use]
    pub fn column_spans(&self) -> Vec<(f64, f64)> {
        self.cells.iter().take(self.cols as usize).map(|c| (c.x, c.x + c.w)).collect()
    }

    /// Each row's `(top, bottom)` in sheet pixels — the first column's cells.
    #[must_use]
    pub fn row_spans(&self) -> Vec<(f64, f64)> {
        self.cells
            .iter()
            .step_by(self.cols.max(1) as usize)
            .take(self.rows as usize)
            .map(|c| (c.y, c.y + c.h))
            .collect()
    }
}

/// `clampInt(v,1,128)` (reference line 27580) applied to a column/row count:
/// non-finite or below 1 becomes 1, above 128 becomes 128. The reference
/// parses a DOM string here; a caller on this side has a number already, so
/// only the clamp half is ported.
#[must_use]
pub fn clamp_grid_count(v: i64) -> u32 {
    v.clamp(1, i64::from(MAX_GRID_COUNT)) as u32
}

/// The grid the slicer cuts on — `gridRect` + `#alSlCols`/`#alSlRows` +
/// `#alSlSpacing`, with the reference's uniform `colF`/`rowF` line fractions
/// (`resetLines`, line 27581: `i/cols`, `j/rows`).
///
/// The reference additionally lets those fractions be *dragged* off uniform,
/// per line. That is pure canvas interaction with no headless equivalent and
/// no engine consequence beyond the fractions themselves, so this port carries
/// the uniform case only; the arithmetic below is written against the
/// fractions rather than against `cols`/`rows` so a future draggable-line UI
/// can supply its own without touching [`compute_cells`].
#[derive(Debug, Clone, PartialEq)]
pub struct SliceGrid {
    pub rect: GridRect,
    pub cols: u32,
    pub rows: u32,
    /// `#alSlSpacing`, in sheet pixels, already through the reference's own
    /// `Math.max(0, parseFloat(...)||0)` guard by construction (see
    /// [`SliceGrid::new`]).
    pub spacing: f64,
}

impl SliceGrid {
    /// Build with the reference's own input guards applied: `cols`/`rows`
    /// through [`clamp_grid_count`], `spacing` through
    /// `Math.max(0, parseFloat(v)||0)` (so `NaN` and negatives both become
    /// `0`).
    #[must_use]
    pub fn new(rect: GridRect, cols: i64, rows: i64, spacing: f64) -> Self {
        SliceGrid {
            rect,
            cols: clamp_grid_count(cols),
            rows: clamp_grid_count(rows),
            spacing: if spacing.is_finite() { spacing.max(0.0) } else { 0.0 },
        }
    }
}

/// Cut `grid` into cell rectangles — the reference's `computeCells()`
/// (line 27590), verbatim.
///
/// **The one thing worth reading twice**: spacing is a *half-gutter applied to
/// interior edges only*, not a pitch. Each cell starts at its uniform division
/// line, moved in by `spacing/2` unless it is the first column/row, and ends at
/// the next division line moved back by `spacing/2` unless it is the last. The
/// consequence is that the outer cells are `spacing/2` **wider** than the
/// interior ones, and the classic "cell = (span − margins − gutters)/n, pitch =
/// cell + gutter" formula (which makes every cell equal) does *not* reproduce
/// the reference's output. Golden fixture `6x4 with spacing 8` pins it: outer
/// cells 508px, interior 504px.
#[must_use]
pub fn compute_cells(grid: &SliceGrid) -> CellGrid {
    let cols = grid.cols as usize;
    let rows = grid.rows as usize;
    let g = grid.spacing;
    let r = grid.rect;
    let mut cells = Vec::with_capacity(cols * rows);
    let mut min_w = f64::INFINITY;
    let mut min_h = f64::INFINITY;
    for row in 0..rows {
        for col in 0..cols {
            let col_f0 = col as f64 / cols as f64;
            let col_f1 = (col + 1) as f64 / cols as f64;
            let row_f0 = row as f64 / rows as f64;
            let row_f1 = (row + 1) as f64 / rows as f64;
            let x0 = r.x + col_f0 * r.w + if col > 0 { g / 2.0 } else { 0.0 };
            let x1 = r.x + col_f1 * r.w - if col < cols - 1 { g / 2.0 } else { 0.0 };
            let y0 = r.y + row_f0 * r.h + if row > 0 { g / 2.0 } else { 0.0 };
            let y1 = r.y + row_f1 * r.h - if row < rows - 1 { g / 2.0 } else { 0.0 };
            let w = x1 - x0;
            let h = y1 - y0;
            min_w = js_min(min_w, w);
            min_h = js_min(min_h, h);
            cells.push(CellRect { col: col as u32, row: row as u32, index: row * cols + col, x: x0, y: y0, w, h });
        }
    }
    CellGrid { cells, cols: grid.cols, rows: grid.rows, min_w, min_h }
}

// ---------------------------------------------------------------------------
// Pixels
// ---------------------------------------------------------------------------

/// `background → transparent`: the reference's chroma key (`this.chroma`,
/// applied by `applyChroma`, line 27603). `color` is the sampled RGB, `tol`
/// the `#alChTol` slider (0..150); a pixel whose squared RGB distance is
/// `<= tol*tol` has its alpha zeroed. Already-transparent pixels are skipped.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChromaKey {
    pub color: [u8; 3],
    pub tol: f64,
}

/// `applyChroma(ctx,w,h)` (reference line 27603), in place. Note the
/// comparison is `<=`, so a pixel at *exactly* the tolerance distance is
/// keyed out — golden-pinned, since `<` would be the more natural guess.
pub fn apply_chroma(img: &mut DecodedImage, key: &ChromaKey) {
    let t2 = key.tol * key.tol;
    let [cr, cg, cb] = key.color;
    for px in img.rgba.chunks_exact_mut(4) {
        if px[3] == 0 {
            continue;
        }
        let dr = f64::from(px[0]) - f64::from(cr);
        let dg = f64::from(px[1]) - f64::from(cg);
        let db = f64::from(px[2]) - f64::from(cb);
        if dr * dr + dg * dg + db * db <= t2 {
            px[3] = 0;
        }
    }
}

/// `isBlank(ctx,w,h)` (reference line 27768): true when **no** pixel has
/// alpha greater than [`BLANK_ALPHA_THRESHOLD`]. An empty buffer is blank,
/// matching the reference's own loop-never-runs case.
#[must_use]
pub fn is_blank(img: &DecodedImage) -> bool {
    !img.rgba.chunks_exact(4).any(|px| px[3] > BLANK_ALPHA_THRESHOLD)
}

/// The integer source rectangle `cropCell` blits — `Math.max(0,Math.round(x))`
/// for the origin and `Math.max(1,Math.round(w))` for the extent (reference
/// lines 27774-27775). Split out from [`crop_cell`] because it is the half a
/// headless harness can golden-verify (the blit itself is `ctx.drawImage`).
#[must_use]
pub fn cell_source_rect(cell: &CellRect) -> (u32, u32, u32, u32) {
    let sx = js_round(cell.x).max(0.0);
    let sy = js_round(cell.y).max(0.0);
    let sw = js_round(cell.w).max(1.0);
    let sh = js_round(cell.h).max(1.0);
    // Every value is >= 0 and finite here (js_round passes NaN through, but
    // `.max` against a finite bound resolves it), so the casts are exact for
    // any sheet a decoder could have produced.
    (sx as u32, sy as u32, sw as u32, sh as u32)
}

/// `cropCell(cell)` (reference line 27773), minus the chroma pass its caller
/// folds in — one cell's pixels lifted out of the sheet at 1:1.
///
/// The reference does this with `ctx.drawImage(sheet, sx,sy,sw,sh, 0,0,sw,sh)`
/// onto a fresh `sw×sh` canvas. A canvas starts fully transparent and
/// `drawImage` clips its *source* rectangle to the source image, so any part
/// of the cell hanging off the edge of the sheet lands as transparent rather
/// than as an error or as wrapped/clamped pixels. That is what this
/// reproduces: a clipped blit onto a zeroed buffer, no resampling (source and
/// destination extents are equal), so the pixels are copied byte for byte.
#[must_use]
pub fn crop_cell(sheet: &DecodedImage, cell: &CellRect) -> DecodedImage {
    let (sx, sy, sw, sh) = cell_source_rect(cell);
    let mut out = vec![0u8; (sw as usize) * (sh as usize) * 4];
    for dy in 0..sh {
        let syy = sy + dy;
        if syy >= sheet.h {
            break;
        }
        let copy_w = sw.min(sheet.w.saturating_sub(sx));
        if copy_w == 0 {
            break;
        }
        let src = ((syy as usize) * (sheet.w as usize) + sx as usize) * 4;
        let dst = (dy as usize) * (sw as usize) * 4;
        let n = copy_w as usize * 4;
        out[dst..dst + n].copy_from_slice(&sheet.rgba[src..src + n]);
    }
    DecodedImage { w: sw, h: sh, rgba: out }
}

/// Crop away fully-transparent borders — **a port-side addition, not a port**
/// (`DCC_SHELL_SPEC.md` §8's *Trim transparent edges*; the reference slicer has
/// no such operation, see the module docs). "Transparent" here means alpha
/// `<=` [`BLANK_ALPHA_THRESHOLD`], the reference's *own* threshold, so trim and
/// [`is_blank`] can never disagree about what content is.
///
/// Returns the image unchanged when there is nothing to trim, and when the
/// image is blank outright — trimming a blank cell to a zero-sized image would
/// produce something no downstream consumer accepts.
#[must_use]
pub fn trim_transparent_edges(img: &DecodedImage) -> DecodedImage {
    let (w, h) = (img.w as usize, img.h as usize);
    let opaque = |x: usize, y: usize| img.rgba[(y * w + x) * 4 + 3] > BLANK_ALPHA_THRESHOLD;
    let Some(min_y) = (0..h).find(|&y| (0..w).any(|x| opaque(x, y))) else {
        return img.clone();
    };
    let max_y = (0..h).rev().find(|&y| (0..w).any(|x| opaque(x, y))).expect("min_y proves one exists");
    let min_x = (0..w).find(|&x| (min_y..=max_y).any(|y| opaque(x, y))).expect("min_y proves one exists");
    let max_x = (0..w).rev().find(|&x| (min_y..=max_y).any(|y| opaque(x, y))).expect("min_x proves one exists");
    if min_x == 0 && min_y == 0 && max_x + 1 == w && max_y + 1 == h {
        return img.clone();
    }
    let (nw, nh) = (max_x - min_x + 1, max_y - min_y + 1);
    let mut out = Vec::with_capacity(nw * nh * 4);
    for y in min_y..=max_y {
        let row = (y * w + min_x) * 4;
        out.extend_from_slice(&img.rgba[row..row + nw * 4]);
    }
    DecodedImage { w: nw as u32, h: nh as u32, rgba: out }
}

// ---------------------------------------------------------------------------
// The whole operation
// ---------------------------------------------------------------------------

/// The slicer's two pixel toggles plus this port's own third one.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SliceOptions {
    /// `#alChEnable` + `#alChTol`: background → transparent.
    pub chroma: Option<ChromaKey>,
    /// §8's *Trim transparent edges* — see [`trim_transparent_edges`], a
    /// port-side addition rather than a ported control.
    pub trim: bool,
    /// `#alSlSkip` (checked by default in the reference): drop cells that
    /// come out blank.
    pub skip_blank: bool,
}

/// One sliced cell: its grid position and its finished pixels.
#[derive(Debug, Clone, PartialEq)]
pub struct SlicedCell {
    pub col: u32,
    pub row: u32,
    pub index: usize,
    /// [`is_blank`] on the cell **after** the chroma pass and **before** any
    /// trim — the reference's own test point, kept there so the verdict stays
    /// golden-exact whether or not this port's trim is on.
    pub blank: bool,
    pub image: DecodedImage,
}

impl SlicedCell {
    /// The reference's default per-cell name for a slot target:
    /// `base+'_r'+(r+1)+'c'+(c+1)` (line 27814), where `base` is the sheet's
    /// filename with its extension stripped.
    #[must_use]
    pub fn default_name(&self, base: &str) -> String {
        format!("{base}_r{}c{}", self.row + 1, self.col + 1)
    }

    /// The reference's default name for the *separate custom icons* target:
    /// `'cell '+(idx+1)` (line 27796).
    #[must_use]
    pub fn default_cell_name(&self) -> String {
        format!("cell {}", self.index + 1)
    }
}

/// Slice `sheet` into cells — `computeCells` + `cropCell` + the chroma and
/// skip-blank passes, in the reference's own order (crop, then chroma, then
/// the blank verdict; line 27810-27813). Cells are returned in `idx` order.
///
/// With `skip_blank` on, blank cells are omitted from the result entirely;
/// with it off they are returned carrying `blank: true`, so a caller can
/// report the count either way. A grid that is too dense to be usable
/// ([`CellGrid::is_usable`]) yields an empty `Vec` rather than a pile of
/// 1×1 crops — the reference refuses to draw it at all in that state.
#[must_use]
pub fn slice_sheet(sheet: &DecodedImage, grid: &SliceGrid, opts: &SliceOptions) -> Vec<SlicedCell> {
    let computed = compute_cells(grid);
    if !computed.is_usable() {
        return Vec::new();
    }
    computed
        .cells
        .iter()
        .filter_map(|cell| {
            let mut img = crop_cell(sheet, cell);
            if let Some(key) = opts.chroma.as_ref() {
                apply_chroma(&mut img, key);
            }
            let blank = is_blank(&img);
            if blank && opts.skip_blank {
                return None;
            }
            if opts.trim && !blank {
                img = trim_transparent_edges(&img);
            }
            Some(SlicedCell { col: cell.col, row: cell.row, index: cell.index, blank, image: img })
        })
        .collect()
}

/// How many cells the current grid produces and how many of them carry
/// content — §8's `24 cells detected · 19 non-empty` readout, computed the
/// way the slice itself would compute it (same crop, same chroma, same
/// [`is_blank`]) rather than sampled. `total` counts every cell in the grid,
/// including blank ones; `usable` is [`CellGrid::is_usable`].
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SliceCounts {
    pub total: usize,
    pub non_empty: usize,
    pub usable: bool,
    /// The `index` of every cell that came out blank, ascending — so a
    /// preview can mark exactly the cells "skip empty cells" would drop,
    /// rather than only saying how many there are.
    pub blank: Vec<usize>,
}

/// Count the grid's cells and note which are blank, without keeping any
/// pixels — the exact detection pass behind §8's readout.
#[must_use]
pub fn count_cells(sheet: &DecodedImage, grid: &SliceGrid, chroma: Option<&ChromaKey>) -> SliceCounts {
    let computed = compute_cells(grid);
    let usable = computed.is_usable();
    if !usable {
        return SliceCounts { total: computed.cells.len(), non_empty: 0, usable, blank: Vec::new() };
    }
    let blank: Vec<usize> = computed
        .cells
        .iter()
        .filter(|cell| {
            let mut img = crop_cell(sheet, cell);
            if let Some(key) = chroma {
                apply_chroma(&mut img, key);
            }
            is_blank(&img)
        })
        .map(|cell| cell.index)
        .collect();
    SliceCounts { total: computed.cells.len(), non_empty: computed.cells.len() - blank.len(), usable, blank }
}

/// Strip a filename's extension the way the reference does before building
/// per-cell names: `name.replace(/\.[^.]+$/,'')` (line 27789). A name with no
/// dot, or one ending in a dot, is returned unchanged — `[^.]+` needs at
/// least one non-dot character after the final dot.
#[must_use]
pub fn sheet_base_name(file_name: &str) -> String {
    match file_name.rfind('.') {
        Some(i) if i + 1 < file_name.len() && !file_name[i + 1..].contains('.') => file_name[..i].to_string(),
        _ => file_name.to_string(),
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    /// A sheet whose every pixel encodes its own coordinates, so a crop can be
    /// checked for having taken the *right* pixels rather than merely the
    /// right number of them.
    fn coord_sheet(w: u32, h: u32) -> DecodedImage {
        let mut rgba = Vec::with_capacity((w * h * 4) as usize);
        for y in 0..h {
            for x in 0..w {
                rgba.extend_from_slice(&[x as u8, y as u8, 0, 255]);
            }
        }
        DecodedImage::new(w, h, rgba).unwrap()
    }

    fn solid(w: u32, h: u32, px: [u8; 4]) -> DecodedImage {
        DecodedImage::new(w, h, std::iter::repeat_n(px, (w * h) as usize).flatten().collect()).unwrap()
    }

    #[test]
    fn clamp_grid_count_matches_clamp_int_1_128() {
        assert_eq!(clamp_grid_count(0), 1);
        assert_eq!(clamp_grid_count(-7), 1);
        assert_eq!(clamp_grid_count(1), 1);
        assert_eq!(clamp_grid_count(128), 128);
        assert_eq!(clamp_grid_count(999), 128);
    }

    #[test]
    fn slice_grid_new_clamps_nan_and_negative_spacing_to_zero() {
        let r = GridRect::whole(64, 64);
        assert_eq!(SliceGrid::new(r, 2, 2, f64::NAN).spacing, 0.0);
        assert_eq!(SliceGrid::new(r, 2, 2, -10.0).spacing, 0.0);
        assert_eq!(SliceGrid::new(r, 2, 2, 7.5).spacing, 7.5);
    }

    #[test]
    fn grid_rect_inset_refuses_a_margin_that_eats_the_sheet() {
        assert!(GridRect::inset(64, 64, 32.0).is_none());
        assert_eq!(GridRect::inset(64, 64, 8.0).unwrap(), GridRect { x: 8.0, y: 8.0, w: 48.0, h: 48.0 });
        // A negative margin is treated as none, not as an outset.
        assert_eq!(GridRect::inset(64, 64, -5.0).unwrap(), GridRect::whole(64, 64));
    }

    #[test]
    fn crop_cell_takes_the_right_pixels_not_merely_the_right_count() {
        let sheet = coord_sheet(16, 16);
        let cell = CellRect { col: 1, row: 1, index: 3, x: 8.0, y: 8.0, w: 8.0, h: 8.0 };
        let out = crop_cell(&sheet, &cell);
        assert_eq!((out.w, out.h), (8, 8));
        assert_eq!(&out.rgba[0..4], &[8, 8, 0, 255], "top-left of the crop is sheet (8,8)");
        let last = out.rgba.len() - 4;
        assert_eq!(&out.rgba[last..], &[15, 15, 0, 255], "bottom-right is sheet (15,15)");
    }

    #[test]
    fn crop_cell_leaves_out_of_bounds_area_transparent() {
        // A cell hanging off the right and bottom edges: drawImage clips the
        // source and leaves the rest of the fresh canvas transparent.
        let sheet = coord_sheet(8, 8);
        let cell = CellRect { col: 0, row: 0, index: 0, x: 6.0, y: 6.0, w: 4.0, h: 4.0 };
        let out = crop_cell(&sheet, &cell);
        assert_eq!((out.w, out.h), (4, 4));
        assert_eq!(&out.rgba[0..4], &[6, 6, 0, 255]);
        // (2,0) in the crop is sheet x=8 -> off the edge.
        assert_eq!(&out.rgba[8..12], &[0, 0, 0, 0]);
        // The whole last row is sheet y=9 -> off the edge.
        assert!(out.rgba[3 * 4 * 4..].iter().all(|&b| b == 0));
    }

    #[test]
    fn crop_cell_entirely_off_the_sheet_is_all_transparent() {
        let sheet = coord_sheet(8, 8);
        let cell = CellRect { col: 0, row: 0, index: 0, x: 100.0, y: 100.0, w: 4.0, h: 4.0 };
        let out = crop_cell(&sheet, &cell);
        assert_eq!(out.rgba.len(), 4 * 4 * 4);
        assert!(out.rgba.iter().all(|&b| b == 0));
    }

    #[test]
    fn trim_removes_transparent_borders_and_keeps_the_content_pixels() {
        // 6x6 with a 3x2 opaque block at (2,1).
        let mut img = solid(6, 6, [0, 0, 0, 0]);
        for y in 1..3u32 {
            for x in 2..5u32 {
                let i = ((y * 6 + x) * 4) as usize;
                img.rgba[i..i + 4].copy_from_slice(&[9, 8, 7, 255]);
            }
        }
        let out = trim_transparent_edges(&img);
        assert_eq!((out.w, out.h), (3, 2));
        assert!(out.rgba.chunks_exact(4).all(|p| p == [9, 8, 7, 255]));
    }

    #[test]
    fn trim_uses_the_references_own_alpha_threshold_not_zero() {
        // An alpha-8 border is "transparent" to `isBlank`, so it must be to
        // trim as well -- the two can never disagree.
        let mut img = solid(3, 1, [0, 0, 0, 8]);
        img.rgba[4 + 3] = 9;
        let out = trim_transparent_edges(&img);
        assert_eq!((out.w, out.h), (1, 1));
        assert_eq!(out.rgba[3], 9);
    }

    #[test]
    fn trim_leaves_a_blank_or_already_tight_image_alone() {
        let blank = solid(4, 4, [0, 0, 0, 0]);
        assert_eq!(trim_transparent_edges(&blank), blank);
        let tight = solid(4, 4, [1, 2, 3, 255]);
        assert_eq!(trim_transparent_edges(&tight), tight);
    }

    #[test]
    fn slice_sheet_skips_blank_cells_only_when_asked() {
        // 4x2 sheet, left half opaque, right half transparent -> 2 cells,
        // one blank.
        let mut sheet = solid(4, 2, [0, 0, 0, 0]);
        for y in 0..2u32 {
            for x in 0..2u32 {
                let i = ((y * 4 + x) * 4) as usize;
                sheet.rgba[i..i + 4].copy_from_slice(&[1, 2, 3, 255]);
            }
        }
        let grid = SliceGrid::new(GridRect::whole(4, 2), 2, 1, 0.0);

        let keep = slice_sheet(&sheet, &grid, &SliceOptions::default());
        assert_eq!(keep.len(), 2);
        assert!(!keep[0].blank && keep[1].blank);

        let skip = slice_sheet(&sheet, &grid, &SliceOptions { skip_blank: true, ..Default::default() });
        assert_eq!(skip.len(), 1);
        assert_eq!(skip[0].index, 0);
    }

    #[test]
    fn slice_sheet_chroma_can_turn_a_cell_blank() {
        // Every pixel is the keyed colour, so after the chroma pass the whole
        // sheet is transparent and both cells drop out.
        let sheet = solid(4, 2, [255, 255, 255, 255]);
        let grid = SliceGrid::new(GridRect::whole(4, 2), 2, 1, 0.0);
        let opts = SliceOptions {
            chroma: Some(ChromaKey { color: [255, 255, 255], tol: 0.0 }),
            skip_blank: true,
            trim: false,
        };
        assert!(slice_sheet(&sheet, &grid, &opts).is_empty());
    }

    #[test]
    fn slice_sheet_refuses_a_too_dense_grid() {
        let sheet = solid(16, 16, [1, 1, 1, 255]);
        let grid = SliceGrid::new(GridRect::whole(16, 16), 4, 4, 100.0);
        assert!(!compute_cells(&grid).is_usable());
        assert!(slice_sheet(&sheet, &grid, &SliceOptions::default()).is_empty());
    }

    #[test]
    fn column_and_row_spans_are_the_overlay_lines_the_slice_actually_cuts_on() {
        // The half-gutter model again, seen from the overlay's side: the two
        // outer columns are wider, so a preview drawn from an equal-pitch
        // formula would sit visibly off the cells the slice produces.
        let grid = SliceGrid::new(GridRect::whole(3072, 2048), 6, 4, 8.0);
        let computed = compute_cells(&grid);
        let cols = computed.column_spans();
        let rows = computed.row_spans();
        assert_eq!(cols.len(), 6);
        assert_eq!(rows.len(), 4);
        assert_eq!(cols[0], (0.0, 508.0));
        assert_eq!(cols[1], (516.0, 1020.0));
        assert_eq!(cols[5], (2564.0, 3072.0));
        assert_eq!(rows[0], (0.0, 508.0));
        assert_eq!(rows[3], (1540.0, 2048.0));
        // Every span matches the cell it was derived from, by construction.
        for (i, span) in cols.iter().enumerate() {
            assert_eq!(*span, (computed.cells[i].x, computed.cells[i].x + computed.cells[i].w));
        }
        for (j, span) in rows.iter().enumerate() {
            let c = &computed.cells[j * 6];
            assert_eq!(*span, (c.y, c.y + c.h));
        }
    }

    #[test]
    fn count_cells_is_the_same_verdict_the_slice_would_reach() {
        let mut sheet = solid(4, 2, [0, 0, 0, 0]);
        sheet.rgba[3] = 255;
        let grid = SliceGrid::new(GridRect::whole(4, 2), 2, 1, 0.0);
        let counts = count_cells(&sheet, &grid, None);
        assert_eq!((counts.total, counts.non_empty, counts.usable), (2, 1, true));
        // The blank list names the cell, not just the count -- and it is
        // exactly the cell `skip_blank` drops.
        assert_eq!(counts.blank, vec![1]);
        let sliced = slice_sheet(&sheet, &grid, &SliceOptions { skip_blank: true, ..Default::default() });
        assert_eq!(sliced.len(), counts.non_empty);
        assert_eq!(sliced[0].index, 0);
    }

    #[test]
    fn default_names_match_the_references_two_conventions() {
        let c = SlicedCell {
            col: 2,
            row: 1,
            index: 6,
            blank: false,
            image: solid(1, 1, [0, 0, 0, 255]),
        };
        assert_eq!(c.default_name("towns-sheet"), "towns-sheet_r2c3");
        assert_eq!(c.default_cell_name(), "cell 7");
    }

    #[test]
    fn sheet_base_name_strips_one_trailing_extension_only() {
        assert_eq!(sheet_base_name("towns-sheet.png"), "towns-sheet");
        assert_eq!(sheet_base_name("a.b.png"), "a.b");
        assert_eq!(sheet_base_name("noext"), "noext");
        assert_eq!(sheet_base_name("trailing."), "trailing.");
    }
}
