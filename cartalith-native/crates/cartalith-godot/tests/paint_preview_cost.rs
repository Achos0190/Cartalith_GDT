//! `OUTSTANDING_WORK.md` §2.6 — *"Previews re-upload the whole texture,
//! `touched_tiles`/`touched_bounds` unused"*, the **paint** half: the
//! measurement that killed the decline, and the correctness proof for the
//! bounded path that replaced it.
//!
//! `build_sculpt_preview_texture`'s decline is already measured and **still
//! stands**: the `SCULPT_LIVE_SCOPE.md` L0 table
//! (`tests/sculpt_live_l0_bench.rs`) breaks its cost down, L1 owns the
//! bounded-window rework, and `render.rs::with_appearance` still runs its
//! whole-grid passes on construction with no window parameter. Nothing in
//! this file touches it. The **paint** preview's decline was never measured
//! at all. Its doc comment argued the saving "is negligible here" because
//! the pass is a flat per-cell lookup with no `RenderCtx` under it — true
//! about the *shape* of the work, and silent about its *size*.
//!
//! What this file now holds, in order: the size (`paint_preview_cost`), the
//! two correctness properties a bounded variant had to hold, the proof that
//! the shipped `PaintEditor::preview_patch` holds them
//! (`the_patch_is_byte_identical_to_a_full_reupload` and
//! `nothing_outside_the_patch_window_moved`), and the win it actually buys
//! (`the_measured_win`).
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
//! bench below times against their window-sized equivalents. The bounded
//! path removes the first, third and fourth outright and keeps the second
//! (a calloc whose pages outside the window are never faulted in).
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
//! its own eight. `render.rs` has no `#[cfg(test)]` module, which is why
//! the five test files that already `#[path]`-include it duplicate
//! nothing.
//!
//! Both benches — `paint_preview_cost` and `the_measured_win` — are
//! `#[ignore]`d (2048² and 4096² allocations, seconds each); the six
//! correctness tests are not, and are cheap. Run the benches with:
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
use paint_bridge::{swatch_color, PaintEditor, PaintTarget};

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
// Mirrors `PaintEditor::preview_full` — which is what
// `WorldGen::build_paint_preview_texture` now calls — split so each stage
// can be timed on its own. Any divergence here would make the stage numbers
// describe something the shell never runs, so these helpers are
// deliberately literal transcriptions rather than a tidier rewrite, and
// `the_stage_transcription_still_matches_the_shipped_body` asserts they add
// up to it.

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

/// The whole full-grid CPU body, FFI excluded — the shipped
/// `PaintEditor::preview_full`, which is what `_paint_apply_dab` pays on
/// every pointer-move sample today.
///
/// `preview_full`'s `None` — *"nothing committed and nothing pending"* — is
/// spelled out here as the fully transparent grid it is equivalent to on
/// screen, because these tests need a same-length "before" raster to diff
/// against. **Only a test may make that collapse.** The binding deliberately
/// does not: `build_paint_preview_patch` returns an empty `Dictionary`, so a
/// caller can still tell "draw nothing" from "draw this transparent
/// rectangle" and does not upload `4n` zero bytes in order to say nothing.
fn full_preview_bytes(p: &PaintEditor, gw: usize, gh: usize) -> Vec<u8> {
    p.preview_full(gw, gh).unwrap_or_else(|| vec![0u8; gw * gh * 4])
}

/// The stage transcriptions above, summed — what the bench times, and what
/// the test of the same name asserts against the shipped body.
fn transcribed_full_bytes(p: &PaintEditor, gw: usize, gh: usize) -> Vec<u8> {
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

/// The stage-by-stage helpers above exist only so the bench can attribute
/// cost; if they drift from `PaintEditor::preview_full` the attribution
/// describes code nothing runs. Asserted rather than assumed, because this
/// file transcribed a body that has since moved into `paint_bridge.rs`.
#[test]
fn the_stage_transcription_still_matches_the_shipped_body() {
    const G: usize = 96;
    for dabs in [0usize, 1, 7] {
        let p = dragged_editor(G, G, dabs);
        match p.preview_full(G, G) {
            // Nothing committed and nothing pending: the shipped body says
            // "nothing to draw" rather than handing back 4n zero bytes.
            None => assert_eq!(dabs, 0, "only the empty editor may decline"),
            Some(shipped) => assert_eq!(shipped, transcribed_full_bytes(&p, G, G), "dabs = {dabs}"),
        }
    }
}

/// Every editor state the two properties below are checked over, so each is
/// checked against a spread of shapes rather than one lucky drag. Named
/// states rather than seeds because a paint draft has no seed: what varies
/// is the layer (three palettes, and `swatch_color` genuinely differs per
/// layer), whether anything is committed under the draft, whether the brush
/// erases, and whether the water gate is on.
fn editor_states(g: usize) -> Vec<(&'static str, PaintEditor)> {
    let mut out: Vec<(&'static str, PaintEditor)> = Vec::new();

    out.push(("biome, 5-dab drag, land-only", dragged_editor(g, g, 5)));

    let mut e = PaintEditor::new(g, g, water_mask(g, g));
    e.set_layer(PaintTarget::Terrain);
    e.set_brush(2, 11.0, 1.0, 0.0, false, false);
    for k in 0..4 {
        e.stroke_at(g as f64 * 0.2 + k as f64 * 3.0, g as f64 * 0.7);
    }
    out.push(("terrain, ungated, small brush", e));

    let mut e = PaintEditor::new(g, g, water_mask(g, g));
    e.set_layer(PaintTarget::Splat);
    e.set_brush(1, 25.0, 0.4, 0.6, false, true);
    e.stroke_at(g as f64 * 0.5, g as f64 * 0.5);
    e.stroke_at(g as f64 * 0.55, g as f64 * 0.52);
    out.push(("splat, feathered edge", e));

    // Committed underneath, then a fresh dab on top: the case where the base
    // is genuinely non-zero, which a fresh editor never exercises.
    let mut e = dragged_editor(g, g, 6);
    e.commit_all(g * g);
    e.set_brush(7, 18.0, 1.0, 0.0, false, true);
    e.stroke_at(g as f64 * 0.3, g as f64 * 0.6);
    out.push(("committed layer + a new dab", e));

    // Erase over committed paint: the dab turns opaque pixels transparent,
    // which is a change the "nothing outside moved" check must still see.
    let mut e = dragged_editor(g, g, 6);
    e.commit_all(g * g);
    e.set_brush(0, 14.0, 1.0, 0.0, true, true);
    e.stroke_at(g as f64 * 0.48, g as f64 * 0.48);
    out.push(("erase over committed paint", e));

    out
}

/// **The property the whole row rests on**, asserted against a full
/// re-upload of the same edit rather than against an eyeball: for every
/// state and every grid size, `preview_patch`'s bytes are identical to
/// `preview_full`'s at the same grid offsets, byte for byte.
#[test]
fn the_patch_is_byte_identical_to_a_full_reupload() {
    for g in [64usize, 128, 300] {
        for (name, p) in editor_states(g) {
            let full = p.preview_full(g, g).unwrap_or_else(|| panic!("{name}: something must be drawable"));
            let patch = p.preview_patch(g, g).unwrap_or_else(|| panic!("{name}: something must be drawable"));
            let r = patch.region;

            // The invariant a consumer may rely on, so no caller ever has to
            // guess a stride or reconstruct a missing rectangle.
            assert_eq!(patch.rgba.len(), r.w * r.h * 4, "{name} @{g}");
            assert!(r.w > 0 && r.h > 0, "{name} @{g}: a present patch is never zero-sized");
            assert!(r.x + r.w <= g && r.y + r.h <= g, "{name} @{g}: the window escapes the grid");

            for y in 0..r.h {
                for x in 0..r.w {
                    let b = (y * r.w + x) * 4;
                    let f = ((r.y + y) * g + r.x + x) * 4;
                    assert_eq!(
                        patch.rgba[b..b + 4],
                        full[f..f + 4],
                        "{name} @{g}: pixel ({}, {}) differs from a full re-upload",
                        r.x + x,
                        r.y + y
                    );
                }
            }
        }
    }
}

/// The other half of the same guarantee, and the half that fails
/// *silently*: nothing outside the window changed, so a caller that
/// repaints only the window leaves no stale pixels behind. Diffed against
/// the same editor with its draft discarded — the raster the caller had on
/// screen before this drag.
#[test]
fn nothing_outside_the_patch_window_moved() {
    const G: usize = 128;
    // `editor_states` is deterministic, so a second call is the same editors;
    // discarding their drafts gives the "before" without needing a clone.
    let after = editor_states(G);
    let mut before = editor_states(G);
    let mut windows_checked = 0usize;

    for (i, (name, p)) in after.iter().enumerate() {
        let r = p.preview_patch(G, G).expect("something must be drawable").region;
        if r.w == G && r.h == G {
            continue; // a full-grid answer has no outside to check
        }
        windows_checked += 1;

        let b = &mut before[i].1;
        b.discard_all();
        let before_bytes = b.preview_full(G, G).unwrap_or_else(|| vec![0u8; G * G * 4]);
        let after_bytes = p.preview_full(G, G).expect("something must be drawable");

        let (mut changed_outside, mut changed_inside) = (0usize, 0usize);
        for y in 0..G {
            for x in 0..G {
                let inside = x >= r.x && x < r.x + r.w && y >= r.y && y < r.y + r.h;
                let idx = (y * G + x) * 4;
                if before_bytes[idx..idx + 4] != after_bytes[idx..idx + 4] {
                    if inside {
                        changed_inside += 1;
                    } else {
                        changed_outside += 1;
                    }
                }
            }
        }
        assert_eq!(changed_outside, 0, "{name}: the window missed pixels the draft changed");
        assert!(changed_inside > 0, "{name}: the draft changed nothing, so this state proves nothing");
    }
    assert!(windows_checked >= 4, "only {windows_checked} states produced a sub-window; the check would be near-vacuous");
}

/// The `MISTAKES.md` absent-value rule at this API's boundary, in the shape
/// that matters here: three states, three distinguishable answers, and the
/// dangerous middle one is why `touched_bounds()`'s `Option` is not
/// forwarded to a caller.
#[test]
fn a_full_grid_patch_and_no_patch_are_never_the_same_answer() {
    const G: usize = 64;
    let n = G * G;

    // (1) Nothing committed, nothing pending -> "draw nothing".
    let fresh = PaintEditor::new(G, G, water_mask(G, G));
    assert_eq!(fresh.preview_patch(G, G), None);
    assert_eq!(fresh.preview_full(G, G), None);

    // (2) Committed layer, empty draft -> `touched_bounds()` is `None`, and
    //     the answer is the WHOLE GRID, not nothing. This is the inversion
    //     the row's second correctness property names.
    let mut p = dragged_editor(G, G, 3);
    p.commit_all(n);
    assert_eq!(p.active_draft().touched_bounds(), None);
    let patch = p.preview_patch(G, G).expect("a committed layer still owes its viewer every pixel");
    assert_eq!(patch.region, Region::new(0, 0, G, G));
    assert_eq!(patch.rgba, p.preview_full(G, G).unwrap());
    assert!(patch.rgba.chunks_exact(4).any(|px| px[3] != 0), "and it is not blank");

    // (3) A live draft -> a strict sub-rectangle. Without this the win is
    //     vacuous: a `preview_patch` that always returned the whole grid
    //     would satisfy (1) and (2) perfectly.
    let mut p = PaintEditor::new(G, G, water_mask(G, G));
    p.set_brush(3, 6.0, 1.0, 0.0, false, true);
    p.stroke_at(30.0, 30.0);
    let r = p.preview_patch(G, G).unwrap().region;
    assert_eq!(r, Region::new(24, 24, 13, 13), "a radius-6 dab at (30, 30) is a 13x13 box");
    assert!(r.w < G && r.h < G);

    // (4) A non-empty draft that touches nothing at all is state (2), not
    //     state (1): every stamp off-grid still leaves the committed layer
    //     to draw.
    let mut p = dragged_editor(G, G, 3);
    p.commit_all(n);
    p.set_brush(4, 5.0, 1.0, 0.0, false, false);
    p.stroke_at(-500.0, -500.0);
    assert!(!p.active_draft().is_empty());
    assert_eq!(p.active_draft().touched_bounds(), None);
    assert_eq!(p.preview_patch(G, G).unwrap().region, Region::new(0, 0, G, G));
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
            std::hint::black_box(transcribed_full_bytes(&p, g, g));
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
            std::hint::black_box(transcribed_full_bytes(&p, 2048, 2048));
        });
        println!("  {dabs:>3} dabs in the draft:  full-grid CPU total {s}");
    }
}

/// The win this row actually buys, measured on the shipped pair rather than
/// projected from a sum of parts: `PaintEditor::preview_full` (what
/// `build_paint_preview_texture` runs) against `PaintEditor::preview_patch`
/// (what `build_paint_preview_patch` runs), one 20-dab drag at the 40-cell
/// brush ceiling, same editor, same grid, median of 7 with min..max.
///
/// CPU only, FFI excluded for the reason this file's module doc gives; the
/// byte counts beside them are exact, being the lengths handed to
/// `PackedByteArray::from` on each path.
#[test]
#[ignore = "allocates at 4096^2 repeatedly; run explicitly with --ignored --nocapture --test-threads=1"]
fn the_measured_win() {
    println!();
    for &g in &[512usize, 1024, 2048, 4096] {
        let p = dragged_editor(g, g, 20);
        let win = p.preview_patch(g, g).expect("the drag must draw something").region;

        let full = time_runs(|| (), |()| {
            std::hint::black_box(p.preview_full(g, g));
        });
        let patch = time_runs(|| (), |()| {
            std::hint::black_box(p.preview_patch(g, g));
        });
        println!("==== {g}x{g} ====");
        println!("  preview_full  (whole grid):  {full}  {:>9} bytes to the FFI", g * g * 4);
        println!("  preview_patch ({}x{} window): {patch}  {:>9} bytes to the FFI", win.w, win.h, win.w * win.h * 4);
        println!(
            "  -> {:.1}x less CPU, {:.1}x fewer bytes",
            full.med_ms / patch.med_ms,
            (g * g) as f64 / (win.w * win.h) as f64
        );
    }
}
