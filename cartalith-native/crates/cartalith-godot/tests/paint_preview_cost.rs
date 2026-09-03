//! `OUTSTANDING_WORK.md` §2.6 — *"Previews re-upload the whole texture,
//! `touched_tiles`/`touched_bounds` unused"*, the **paint** half.
//!
//! `build_sculpt_preview_texture`'s decline is already measured: the
//! `SCULPT_LIVE_SCOPE.md` L0 table (`tests/sculpt_live_l0_bench.rs`) breaks
//! its cost down, and L1 owns the bounded-window rework. The **paint**
//! preview's decline was never measured at all. Its doc comment argued the
//! saving "is negligible here" because the pass is a flat per-cell lookup
//! with no `RenderCtx` under it — true about the *shape* of the work, and
//! silent about its *size*. This file supplies the size, and the two
//! correctness properties any bounded variant would have to hold.
//!
//! Two things make the paint preview a different question from the sculpt
//! one, and both are why "negligible" needed a number rather than an
//! argument:
//!
//! 1. **Call frequency.** `world_workspace.gd`'s `_sculpt_release` calls
//!    the sculpt preview once per finished stroke; `_paint_apply_dab` calls
//!    the paint preview from `_paint_click` **and** `_paint_drag`, i.e.
//!    once per pointer-move sample of a continuous drag.
//! 2. **Footprint.** `PAINT_RADIUS_RANGE`'s ceiling is 40 cells, so one dab
//!    can touch at most an 81x81 box — 0.16% of a 2048² grid. Every
//!    grid-proportional byte the preview moves is moved for a region three
//!    orders of magnitude larger than the edit.
//!
//! `PaintStamp::apply` already iterates only `-r..=r` (verified at the
//! symbol, `cartalith-spatial/src/paint.rs`), so it is footprint-bounded in
//! both a full-grid and a bounded path and cancels out of the comparison.
//! Exactly four stages scale with the grid: the committed layer's `to_vec`,
//! the scratch `vec![0u8; n]`, `preview_into`'s `clone_from_slice`, and the
//! per-cell swatch loop that packs `4n` RGBA bytes. Those four are what the
//! bench below times against their window-sized equivalents.
//!
//! **Not measured here:** `PackedByteArray::from` /
//! `Image::create_from_data` / `ImageTexture::create_from_image`. Those
//! cross the GDExtension boundary into a running Godot process, the same
//! limit `sculpt_live_l0_bench.rs`'s module doc records; they move a
//! further `4n` bytes each and can only make the full-grid path worse
//! relative to a bounded one, never better.
//!
//! `#[path]`-includes `render.rs` and `paint_bridge.rs` for the reason
//! every other test in this directory does: `cartalith-godot` is
//! `cdylib`-only (`ARCHITECTURE.md`) with no `rlib` to link against.
//! `paint_bridge::swatch_color` reads `crate::render`'s colour tables, so
//! both modules are needed even though only the paint one is under test.
//! One consequence worth stating so the workspace total is not a mystery:
//! `paint_bridge.rs` carries its own `#[cfg(test)]` suite, and `#[path]`
//! compiles it into this binary too, so this file adds **28 re-run
//! `paint_bridge::tests::*` cases** to `cargo test --workspace` on top of
//! its own three. `render.rs` has no `#[cfg(test)]` module, which is why
//! the five test files that already `#[path]`-include it duplicate
//! nothing.
//!
//! The bench is `#[ignore]`d (2048² allocations, seconds); the two
//! correctness tests are not, and are cheap. Run the bench with:
//! ```text
//! cargo test --release -p cartalith-godot --test paint_preview_cost -- --ignored --nocapture --test-threads=1
//! ```
#![allow(dead_code)]

#[path = "../src/render.rs"]
mod render;
#[path = "../src/paint_bridge.rs"]
mod paint_bridge;

use std::sync::Arc;
use std::time::Instant;

use cartalith_spatial::Region;
use paint_bridge::{swatch_color, PaintEditor};

// ---- fixtures ----

/// A deterministic land/water gate roughly a third water, in coherent bands
/// rather than per-cell noise — `PaintStamp`'s `land_only` mask rejects
/// whole neighbourhoods in a real world, and a salt-and-pepper mask would
/// make every dab touch a different number of cells for reasons no real map
/// has.
fn water_mask(gw: usize, gh: usize) -> Arc<[u8]> {
    let mut m = vec![0u8; gw * gh];
    for y in 0..gh {
        for x in 0..gw {
            let fx = x as f64 / gw as f64;
            let fy = y as f64 / gh as f64;
            let v = (fx * 6.0).sin() + (fy * 4.0).cos() * 0.7;
            m[y * gw + x] = u8::from(v > 0.55);
        }
    }
    m.into()
}

/// A realistic drag: `dabs` samples along a short diagonal near the middle
/// of the grid, at the brush ceiling (40 cells) so the footprint is the
/// largest a caller can actually produce.
fn dragged_editor(gw: usize, gh: usize, dabs: usize) -> PaintEditor {
    let mut e = PaintEditor::new(gw, gh, water_mask(gw, gh));
    e.set_brush(3, 40.0, 1.0, 0.0, false, true);
    for k in 0..dabs {
        let t = k as f64 / dabs.max(1) as f64;
        e.stroke_at(gw as f64 * (0.45 + 0.10 * t), gh as f64 * (0.45 + 0.10 * t));
    }
    e
}

// ---- the shipped body, stage by stage ----
//
// Mirrors `WorldGen::build_paint_preview_texture` in `lib.rs`, split so each
// stage can be timed on its own. Any divergence here would make the numbers
// describe something the shell never runs, so these four helpers are
// deliberately literal transcriptions rather than a tidier rewrite.

fn stage_base(p: &PaintEditor, n: usize) -> Vec<u8> {
    p.active_layer().cells().map(<[u8]>::to_vec).unwrap_or_else(|| vec![0u8; n])
}

fn stage_composite(p: &PaintEditor, base: &[u8], scratch: &mut [u8]) {
    p.active_draft().preview_into(base, scratch);
}

fn stage_pack(p: &PaintEditor, scratch: &[u8]) -> Vec<u8> {
    let palette_len = p.layer.palette().len();
    let mut bytes = Vec::with_capacity(scratch.len() * 4);
    for &v in scratch {
        if v == 0 {
            bytes.extend_from_slice(&[0, 0, 0, 0]);
        } else {
            let (r, g, b) = swatch_color(p.layer, v, palette_len);
            bytes.extend_from_slice(&[r, g, b, 255]);
        }
    }
    bytes
}

/// The whole full-grid CPU body, FFI excluded — what `_paint_apply_dab`
/// pays on every pointer-move sample today.
fn full_preview_bytes(p: &PaintEditor, gw: usize, gh: usize) -> Vec<u8> {
    let n = gw * gh;
    let base = stage_base(p, n);
    let mut scratch = vec![0u8; n];
    stage_composite(p, &base, &mut scratch);
    stage_pack(p, &scratch)
}

/// The same RGBA bytes, for the sub-rectangle `win` only — what a bounded
/// path would upload, given the window to blit it at.
///
/// The composite is read out of a full-grid scratch rather than recomputed:
/// `PaintStamp::apply` is footprint-bounded already, so a real bounded path
/// pays the identical stamp cost and differs from this only in bookkeeping.
/// What this function exists to establish is that the *pixels* agree.
fn window_bytes(p: &PaintEditor, scratch: &[u8], gw: usize, win: Region) -> Vec<u8> {
    let palette_len = p.layer.palette().len();
    let mut bytes = Vec::with_capacity(win.w * win.h * 4);
    for y in win.y..win.y + win.h {
        for &v in &scratch[y * gw + win.x..y * gw + win.x + win.w] {
            if v == 0 {
                bytes.extend_from_slice(&[0, 0, 0, 0]);
            } else {
                let (r, g, b) = swatch_color(p.layer, v, palette_len);
                bytes.extend_from_slice(&[r, g, b, 255]);
            }
        }
    }
    bytes
}

// ---- correctness, before speed ----

/// The property any bounded upload must hold, and the one that fails
/// silently if it does not: the window contains **every** pixel the edit
/// changed, and inside it the bounded raster is byte-identical to the
/// full-grid one. Asserted against a full re-render of the same edit, not
/// against an eyeball.
#[test]
fn touched_bounds_contains_every_pixel_a_dab_changes() {
    const G: usize = 256;
    let n = G * G;

    let before = full_preview_bytes(&dragged_editor(G, G, 0), G, G);
    assert_eq!(before.len(), n * 4);

    let p = dragged_editor(G, G, 5);
    let win = p.active_draft().touched_bounds().expect("five dabs must touch something");
    let after = full_preview_bytes(&p, G, G);

    // The window is genuinely a window, not the whole map — otherwise this
    // test would pass for a `touched_bounds` that gave up and returned
    // everything.
    assert!(win.w < G && win.h < G, "expected a sub-rectangle, got {}x{}", win.w, win.h);

    // (1) Outside the window: nothing moved. This is the assertion that
    //     catches an under-reported dirty region — the failure mode where a
    //     bounded preview shows stale pixels forever.
    let mut changed_outside = 0usize;
    for y in 0..G {
        for x in 0..G {
            let inside = x >= win.x && x < win.x + win.w && y >= win.y && y < win.y + win.h;
            let i = (y * G + x) * 4;
            if !inside && before[i..i + 4] != after[i..i + 4] {
                changed_outside += 1;
            }
        }
    }
    assert_eq!(changed_outside, 0, "touched_bounds missed changed pixels outside its window");

    // (2) Inside the window: the bounded raster equals the full one, byte
    //     for byte.
    let base = stage_base(&p, n);
    let mut scratch = vec![0u8; n];
    stage_composite(&p, &base, &mut scratch);
    let bounded = window_bytes(&p, &scratch, G, win);
    assert_eq!(bounded.len(), win.w * win.h * 4);
    for y in 0..win.h {
        for x in 0..win.w {
            let b = (y * win.w + x) * 4;
            let f = ((win.y + y) * G + win.x + x) * 4;
            assert_eq!(bounded[b..b + 4], after[f..f + 4], "pixel ({}, {}) differs", win.x + x, win.y + y);
        }
    }

    // (3) And the edit was real — a test where nothing changed would pass
    //     (1) and (2) trivially.
    assert_ne!(before, after, "the five dabs must have changed some pixel");
}

/// The `MISTAKES.md` "absent value" rule in its most dangerous form for this
/// consumer. `touched_bounds()` returns `None` for *"the draft touched
/// nothing"* — which is **not** the same as *"nothing needs drawing"*, and
/// is the exact opposite of *"everything is dirty"*.
///
/// A committed paint layer with an empty draft is precisely that case: the
/// preview must still show the whole committed layer, and `touched_bounds()`
/// says `None`. A bounded path that read `None` as "upload nothing" would
/// blank a painted map; one that read it as "upload everything" would be
/// correct here and wasteful after a discard. Neither reading is derivable
/// from the `Option` alone, so this test pins the distinction rather than
/// leaving it to a future reader.
#[test]
fn none_from_touched_bounds_never_means_the_whole_grid() {
    const G: usize = 64;
    let n = G * G;

    let mut p = dragged_editor(G, G, 3);
    assert!(p.active_draft().touched_bounds().is_some());

    p.commit_all(n);

    // Committed: the draft is empty, so `touched_bounds()` is `None` ...
    assert_eq!(p.active_draft().touched_bounds(), None);
    // ... while the preview it feeds is emphatically not empty.
    let bytes = full_preview_bytes(&p, G, G);
    let painted = bytes.chunks_exact(4).filter(|px| px[3] != 0).count();
    assert!(painted > 0, "a committed layer must still render opaque cells");

    // And the empty-everything case really does produce nothing to draw, so
    // the two `None`s above are told apart by the layer, never by the bounds.
    let fresh = PaintEditor::new(G, G, water_mask(G, G));
    assert_eq!(fresh.active_draft().touched_bounds(), None);
    assert!(fresh.active_layer().is_empty() && fresh.active_draft().is_empty());
}

// ---- the measurement ----

const RUNS: usize = 7;

struct Stats {
    med_ms: f64,
    min_ms: f64,
    max_ms: f64,
}

impl std::fmt::Display for Stats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:8.3} ms  ({:.3}..{:.3}, n={RUNS})", self.med_ms, self.min_ms, self.max_ms)
    }
}

/// One warm-up, then `RUNS` timed samples; median with min..max, never a
/// point estimate (`MISTAKES.md`, "single-sample timings written as measured
/// fact").
fn time_runs<S>(mut setup: impl FnMut() -> S, mut op: impl FnMut(S)) -> Stats {
    op(setup());
    let mut s = [0f64; RUNS];
    for slot in &mut s {
        let x = setup();
        let t0 = Instant::now();
        op(x);
        *slot = t0.elapsed().as_secs_f64() * 1000.0;
    }
    s.sort_by(f64::total_cmp);
    Stats { med_ms: s[RUNS / 2], min_ms: s[0], max_ms: s[RUNS - 1] }
}

#[test]
#[ignore = "allocates at 2048^2 repeatedly; run explicitly with --ignored --nocapture --test-threads=1"]
fn paint_preview_cost() {
    // `new_world_dialog.gd`'s own `RESOLUTION_PRESETS` are 512 / 1K / 2K /
    // 4K / 8K with **2K the default**, so the middle of this list is what a
    // caller who changed nothing gets — not the extreme. 8192² is left out
    // only because one `stage_pack` at that size allocates 268 MB and this
    // bench runs it seven times; its cost is the 4096² row times four, and
    // is quoted as an extrapolation rather than a measurement wherever it
    // appears.
    for &g in &[512usize, 1024, 2048, 4096] {
        let n = g * g;
        println!("\n==== {g}x{g} ({n} cells) ====");

        // A 20-sample drag: what one short gesture leaves in the draft.
        let p = dragged_editor(g, g, 20);
        let win = p.active_draft().touched_bounds().expect("the drag must touch something");
        println!(
            "  touched_bounds: {}x{} at ({}, {}) = {} cells, {:.4}% of the grid",
            win.w,
            win.h,
            win.x,
            win.y,
            win.w * win.h,
            100.0 * (win.w * win.h) as f64 / n as f64
        );

        let base = stage_base(&p, n);
        let mut scratch = vec![0u8; n];
        stage_composite(&p, &base, &mut scratch);

        println!(" -- full-grid stages, one per pointer-move sample --");
        let s = time_runs(|| (), |()| {
            std::hint::black_box(stage_base(&p, n));
        });
        println!("  committed layer to_vec:      {s}");

        let s = time_runs(|| (), |()| {
            std::hint::black_box(vec![0u8; n]);
        });
        println!("  scratch vec![0u8; n]:        {s}");

        let s = time_runs(|| vec![0u8; n], |mut sc| stage_composite(&p, &base, &mut sc));
        println!("  preview_into (copy + stamps):{s}");

        let s = time_runs(|| (), |()| {
            std::hint::black_box(stage_pack(&p, &scratch));
        });
        println!("  swatch loop + RGBA pack:     {s}");

        let s = time_runs(|| (), |()| {
            std::hint::black_box(full_preview_bytes(&p, g, g));
        });
        println!("  FULL-GRID CPU TOTAL:         {s}  (FFI upload of {} bytes excluded)", n * 4);

        println!(" -- the same work over touched_bounds() --");
        let s = time_runs(|| (), |()| {
            std::hint::black_box(window_bytes(&p, &scratch, g, win));
        });
        println!("  window swatch loop + pack:   {s}  ({} bytes)", win.w * win.h * 4);

        // The stamp applies are footprint-bounded already and are paid by
        // both paths; timed here so a bounded total is a real sum rather
        // than an omission.
        let s = time_runs(
            || base.clone(),
            |mut sc| {
                for e in p.active_draft().entries() {
                    cartalith_spatial::Stamp::apply(&e.stamp, &mut sc, g, g);
                }
            },
        );
        println!("  stamp applies (both paths):  {s}");
    }

    // The term a bounded window cannot remove. `preview_into` replays the
    // **whole** draft stack on every call, and every dab appends one more
    // stamp to it, so a long gesture pays O(dabs) stamp applies per sample
    // on top of the O(grid) work above — a cost that is already footprint-
    // bounded and would survive the bounding unchanged.
    println!("\n==== 2048x2048, cost against draft depth ====");
    for &dabs in &[1usize, 20, 100, 300] {
        let p = dragged_editor(2048, 2048, dabs);
        let s = time_runs(|| (), |()| {
            std::hint::black_box(full_preview_bytes(&p, 2048, 2048));
        });
        println!("  {dabs:>3} dabs in the draft:  full-grid CPU total {s}");
    }
}
