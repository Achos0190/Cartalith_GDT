//! Rule-driven icon placement — *where* a scattered asset lands on the map.
//!
//! Ported from `Cartalith Gen1 v2.10.html`: [`place_map_icons_ruled`] is
//! `placeMapIconsRuled` (lines 7194-7283), [`icon_slot_for_item`] plus its
//! `TREE_SLOT`/`SCATTER_SLOT` legacy maps are lines 7284-7300, and
//! [`sprite_draw_rect`] is `spriteDrawRect` (line 12173).
//!
//! This is the first milestone in `cartalith-assets` with real golden-parity
//! **placement** surface: [`place_map_icons_ruled`] is positional and seeded
//! (every random draw is `hash(x, y, seed±k)` on a cell coordinate), so a port
//! either lands icons on the identical cells with the identical sizes, or it
//! does not — there is no "close enough" the way there sometimes is for a
//! continuous field. `tests/golden_parity_placement.rs` verifies it against
//! the real reference.
//!
//! # The legacy engine is out of scope here
//!
//! The reference's `placeMapIcons` (line 7102) runs the ruled engine only
//! when `opts.rules` is non-empty; its own hard-coded body (mountains/hills/
//! trees/scatter via a biome `switch`) is untouched v1.25 code and is not
//! ported by this milestone — nothing in `ASSET_LIBRARY_SCOPE.md`'s milestone
//! 4 scope calls for it, and [`current_scatter_rules`] already reproduces the
//! *conditions* under which the reference would fall through to it (an empty
//! rule table). [`icon_slot_for_item`] is still ported in full, including its
//! legacy `cat`/`kind` branches, because it is the one place a legacy-shaped
//! item (were one ever produced) and a rule-driven item agree on slot
//! spelling — the reference itself keeps `iconSlotForItem` free of the ruled
//! engine's own internals for exactly that reason.
//!
//! # The two v1.27 fixes this milestone owns
//!
//! Both live inside `placeMapIconsRuled`'s scatter branch (reference lines
//! 7250-7280), not in the rule model (`scatter.rs` already carries the other
//! three v1.27 fixes, plus the `spacing`/density hardening this function
//! calls into via [`ScatterRule::spacing_cells`]).
//!
//! 1. **Priority sort — most-specific rule wins a cell, not whichever rule
//!    happened to be inserted first.** Before v1.27, candidate order for a
//!    contested cell was "whatever order the caller happened to build the
//!    array in" — and since the table comes from iterating an object, that
//!    was really "whichever order the user added assets to the Library in".
//!    Two assets both matching a cell could silently swap which one appeared
//!    depending on unrelated editing history. The fix sorts by
//!    [`specificity`] (fewest matching biomes = most specific; a
//!    wetland-requiring rule's contribution is offset below a non-wetland
//!    rule's; an empty biome list — "any land" — sorts last) before the
//!    "first match wins, `break`" loop ever runs. **Structurally necessary in
//!    Rust too**: nothing about ownership or types makes insertion-order
//!    dependence go away — a `Vec` iterates in whatever order it was built,
//!    same as a JS array. This is ported as a real sort, verified by
//!    `tests/golden_parity_placement.rs`'s `v1_27_fix_proof` case: two rules
//!    that both match one cell, deliberately inserted with the *less*
//!    specific one first, are proven to make the winner the *more* specific
//!    one regardless — and the outcome doesn't flip when the insertion order
//!    is reversed.
//! 2. **`requireWetland` is ANDed with the biome test, not substituted for
//!    it.** v1.26's scatter branch had `requireWetland` *replace* `biomeOk`
//!    outright, so a rule with both a biome list and `requireWetland` ticked
//!    silently discarded the user's biome selection — any wetland cell
//!    matched, regardless of biome. v1.27 ANDs the two tests. **Structurally
//!    necessary in Rust too**: this is an algorithm/predicate defect, not a
//!    consequence of JS's type coercion or object semantics the way two of
//!    `scatter.rs`'s three fixes were — a straight transcription of the old
//!    "replace" logic would reproduce the bug faithfully in any language.
//!    Same `v1_27_fix_proof` test: a rule requiring both wetland and a
//!    specific biome is proven to reject a wetland cell of the *wrong*
//!    biome, which the pre-v1.27 replace-semantics would have accepted.
//!
//! Relief mode already ANDs `requireWetland` with the biome test on both
//! sides of v1.27 (reference line 7239) — only the scatter branch had the bug
//! to fix.

use crate::scatter::ScatterRule;
use cartalith_noise::hash;

// ---------------------------------------------------------------------------
// placeMapIconsRuled
// ---------------------------------------------------------------------------

/// Options for [`place_map_icons_ruled`] — the reference's
/// `Object.assign({sea:0.42, seed:7, tGap:Math.max(4,Math.round(W/110)),
/// tempField:null, wetlandMask:null, rules:[]}, opts||{})`.
///
/// `tempField` is in the reference's own default object but is never read
/// anywhere in `placeMapIconsRuled`'s body (only `wetlandMask` is), so it is
/// not carried here — there is nothing to port.
///
/// `t_gap`'s reference default depends on `W`, so unlike `sea`/`seed` it has
/// no meaningful value before a grid width is known; use [`Self::new`] to get
/// the reference's own default, then override individual fields.
pub struct PlaceIconsRuledOpts<'a> {
    /// Sea level: cells at or below this in `fld` are never candidates.
    pub sea: f64,
    /// The seed all placement hashing derives from.
    pub seed: i32,
    /// Jittered-grid spacing (in cells) for the scatter pass. Must be at
    /// least 1: unlike the reference (where `tGap<=0` would simply hang the
    /// `for(;gy+=g)` loop forever), a Rust `step_by(0)` panics, so this is
    /// clamped to 1 rather than reproduced — a different failure mode for an
    /// input malformed the same way, not a behavioural change for any input
    /// that ever reaches this function through [`Self::new`].
    pub t_gap: usize,
    /// `wm[i]===1` in the reference: `Some(mask)` where `mask[i] == 1` means
    /// cell `i` is wetland. `None` matches an absent `opts.wetlandMask`.
    pub wetland_mask: Option<&'a [u8]>,
    /// `(key, rule)` pairs — the reference's `opts.rules`, an array of rule
    /// objects each carrying its own `key` (`Object.assign({key}, r)` in
    /// `currentScatterRules`). [`crate::current_scatter_rules`] produces
    /// exactly this shape already filtered to enabled rules; this function
    /// filters again regardless (`r=>r&&r.enabled!==false`), matching the
    /// reference's own defensive re-check.
    pub rules: &'a [(&'a str, &'a ScatterRule)],
}

impl<'a> PlaceIconsRuledOpts<'a> {
    /// The reference's own defaults, `t_gap` resolved against `map_width`
    /// (`Math.max(4, Math.round(W/110))`).
    pub fn new(map_width: usize, rules: &'a [(&'a str, &'a ScatterRule)]) -> Self {
        PlaceIconsRuledOpts {
            sea: 0.42,
            seed: 7,
            t_gap: ((map_width as f64 / 110.0).round() as usize).max(4),
            wetland_mask: None,
            rules,
        }
    }
}

/// One placed icon — the reference's item shape as `placeMapIconsRuled`
/// produces it: always `cat: 'ruled'` with an explicit `key`. `x`/`y` are grid
/// cell coordinates; `s` is the size multiplier `drawMapIcons`/
/// `spriteDrawRect` scale a sprite by.
#[derive(Debug, Clone, PartialEq)]
pub struct PlacedIcon {
    pub x: i32,
    pub y: i32,
    pub s: f64,
    /// The reference's `it.key`. `Some("")` (an empty key) is treated as
    /// absent by [`icon_slot_for_item`], matching JS's `if(it.key)` falsy
    /// test — [`place_map_icons_ruled`] itself never produces one, since
    /// [`crate::scatter_rule_key`] never returns an empty string.
    pub key: Option<String>,
    /// The reference's `it.cat`. Every item this module's own placement
    /// engine produces is [`IconCategory::Ruled`]; the other variants exist
    /// so [`icon_slot_for_item`] can resolve a legacy-shaped item too (see
    /// the module docs on why the legacy generator itself is out of scope).
    pub cat: IconCategory,
    /// The reference's `it.kind`, meaningful only for [`IconCategory::Tree`]
    /// and [`IconCategory::Scatter`] items.
    pub kind: Option<IconKind>,
}

/// The reference's `it.cat` values across both the legacy and ruled engines.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IconCategory {
    Mountain,
    Hill,
    Tree,
    Scatter,
    /// `placeMapIconsRuled`'s own category. Always paired with a non-empty
    /// `key` in practice, which is why the fallback branches in
    /// [`icon_slot_for_item`]'s match on this variant are unreachable from
    /// this crate's own placement engine — they exist only because the
    /// reference's `if(it.key)` check comes first and a hypothetical
    /// legacy-shaped caller could still reach them.
    Ruled,
}

/// The reference's `it.kind` values, merging `TREE_SLOT`'s and
/// `SCATTER_SLOT`'s keys into one enum — safe because [`tree_slot`] and
/// [`scatter_slot`] each only ever look up the variants their own map
/// defines, falling back for the rest exactly as `TREE_SLOT[kind]||'…'` and
/// `SCATTER_SLOT[kind]||'…'` do for a kind their own object doesn't hold.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IconKind {
    Conifer,
    Broadleaf,
    Rainforest,
    Savanna,
    Wetland,
    Shrub,
    Cactus,
    Boulder,
}

/// `placeMapIconsRuled` (reference line 7194). Reached only through this
/// function — the reference's own legacy `placeMapIcons` gates entry to it on
/// `opts.rules` being non-empty, but that gate is the *caller*'s concern (see
/// the module docs); this function always runs the ruled engine.
///
/// `fld` is the elevation field (row-major, `w*h`); `biome` is the per-cell
/// `BIOME_INDEX` raster (`None` matches the reference's `!biome`, under which
/// every rule with a non-empty `biomes` list rejects every cell — see
/// [`biome_ok`]).
///
/// The reference returns `{mountains:[], hills:[], trees:[], scatter:[],
/// items}` — the first four are always empty from this path (only the legacy
/// engine ever populates them) and exist solely so old callers reading that
/// shape don't break; nothing in this port reads them, so only `items` is
/// returned.
pub fn place_map_icons_ruled(
    fld: &[f64],
    biome: Option<&[u8]>,
    w: usize,
    h: usize,
    opts: &PlaceIconsRuledOpts,
) -> Vec<PlacedIcon> {
    if w == 0 || h == 0 {
        // A Rust-specific guard, not a reference behaviour: the reference's
        // `W-1`/`H-1` indexing on an empty grid is still valid JS (produces
        // `-1`, which every subsequent array access on an empty `fld`/`biome`
        // would already fail to read meaningfully). `usize` has no negative
        // value to underflow to, so this returns early rather than panicking
        // on `w - 1` below.
        return Vec::new();
    }
    let sea = opts.sea;
    // `(1-sea)||1`: JS falsy only for exactly 0 (or NaN, not a concern for a
    // direct numeric parameter the way an untrusted-JSON field is).
    let land_den = {
        let d = 1.0 - sea;
        if d == 0.0 { 1.0 } else { d }
    };
    let wm = opts.wetland_mask;
    let rules: Vec<(&str, &ScatterRule)> = opts
        .rules
        .iter()
        .copied()
        .filter(|(_, r)| r.enabled)
        .collect();
    let mut items: Vec<PlacedIcon> = Vec::new();

    // ---- relief rules: shared spacing grid, highest elevation band first ----
    let mut relief: Vec<(&str, &ScatterRule)> = rules
        .iter()
        .copied()
        .filter(|(_, r)| r.mode == crate::scatter::ScatterMode::Relief)
        .collect();
    // `(b.elevMin==null?-1:b.elevMin)-(a.elevMin==null?-1:a.elevMin)`,
    // descending. The reference's sort is stable (ES2019+); so is Rust's.
    relief.sort_by(|a, b| {
        let ea = a.1.elev_min.unwrap_or(-1.0);
        let eb = b.1.elev_min.unwrap_or(-1.0);
        eb.partial_cmp(&ea).expect("elev_min is never NaN")
    });
    if !relief.is_empty() {
        let cell = relief
            .iter()
            .map(|(_, r)| r.spacing_cells(w))
            .fold(f64::NEG_INFINITY, f64::max);
        let bw = ((w as f64 / cell).ceil() as usize).max(1);
        let bh = ((h as f64 / cell).ceil() as usize).max(1);
        let mut buckets: Vec<Vec<(f64, f64)>> = vec![Vec::new(); bw * bh];

        let lo_band = relief
            .iter()
            .map(|(_, r)| r.elev_min.unwrap_or(0.0))
            .fold(f64::INFINITY, f64::min);
        let mut cand: Vec<(i32, i32, f64)> = Vec::new();
        for y in 0..h {
            for x in 0..w {
                let v = fld[y * w + x];
                if v <= sea {
                    continue;
                }
                let r = (v - sea) / land_den;
                if r >= lo_band {
                    cand.push((x as i32, y as i32, r));
                }
            }
        }
        cand.sort_by(|a, b| b.2.partial_cmp(&a.2).expect("fld is never NaN"));

        for (key, rule) in &relief {
            let lo = rule.elev_min.unwrap_or(0.0);
            let hi = rule.elev_max.unwrap_or(f64::INFINITY);
            let space = rule.spacing_cells(w);
            for &(cx, cy, cr) in &cand {
                if cr < lo || cr >= hi {
                    continue;
                }
                let i = cy as usize * w + cx as usize;
                if !biome_ok(rule, biome, i) {
                    continue;
                }
                if rule.require_wetland && !wetland_at(wm, i) {
                    continue;
                }
                if !relief_fits(&buckets, bw, bh, cell, cx as f64, cy as f64, space) {
                    continue;
                }
                // size tracks position within the rule's own elevation band,
                // so the tallest peaks stay the biggest sprites.
                let span = (if hi.is_infinite() { 1.0 } else { hi }) - lo;
                let f = if span > 0.0 {
                    ((cr - lo) / span).min(1.0)
                } else {
                    0.5
                };
                let s = size_at(rule, cx, cy, opts.seed, Some(f));
                items.push(PlacedIcon {
                    x: cx,
                    y: cy,
                    s,
                    key: Some((*key).to_string()),
                    cat: IconCategory::Ruled,
                    kind: None,
                });
                relief_take(&mut buckets, bw, cell, cx as f64, cy as f64);
            }
        }
    }

    // ---- scatter rules: jittered grid, one visit per cell, most-specific wins ----
    let mut scat: Vec<(&str, &ScatterRule)> = rules
        .iter()
        .copied()
        .filter(|(_, r)| r.mode != crate::scatter::ScatterMode::Relief)
        .collect();
    scat.sort_by_key(|(_, r)| specificity(r));
    if !scat.is_empty() {
        let g = opts.t_gap.max(1);
        let mut gy = 0usize;
        while gy < h {
            let mut gx = 0usize;
            while gx < w {
                let jx = (gx + ((hash(gx as i32, gy as i32, opts.seed) * g as f64) as usize))
                    .min(w - 1);
                let jy = (gy
                    + ((hash(gx as i32, gy as i32, opts.seed + 1) * g as f64) as usize))
                    .min(h - 1);
                let i = jy * w + jx;
                if fld[i] > sea {
                    let keep = hash(jx as i32, jy as i32, opts.seed + 3);
                    for (key, rule) in &scat {
                        // v1.27 fix: requireWetland ANDed with the biome
                        // test in this branch too (see module docs).
                        if rule.require_wetland && !wetland_at(wm, i) {
                            continue;
                        }
                        if !biome_ok(rule, biome, i) {
                            continue;
                        }
                        if keep >= rule.density.min(1.0) {
                            continue;
                        }
                        let s = size_at(rule, jx as i32, jy as i32, opts.seed, None);
                        items.push(PlacedIcon {
                            x: jx as i32,
                            y: jy as i32,
                            s,
                            key: Some((*key).to_string()),
                            cat: IconCategory::Ruled,
                            kind: None,
                        });
                        break; // one asset per cell
                    }
                }
                gx += g;
            }
            gy += g;
        }
    }

    items.sort_by_key(|it| it.y); // unified painter's order
    items
}

/// `biomeOk` (inline in `placeMapIconsRuled`): an empty `rule.biomes` accepts
/// any land cell; otherwise `biome` must be present and `biome[i]` must equal
/// one of the rule's entries **exactly**.
///
/// The cast to `f64` is deliberate, not incidental: [`ScatterRule::biomes`]
/// is `Vec<f64>` because the reference's own filter (`Number.isFinite`, which
/// does not coerce) can leave a non-integer value like `5.5` in the list — it
/// is simply never equal to any `biome[i]`, which is always a plain
/// `BIOME_INDEX` integer. Comparing as `i32` instead would make a `5.5`
/// silently start matching biome `5`.
fn biome_ok(rule: &ScatterRule, biome: Option<&[u8]>, i: usize) -> bool {
    if rule.biomes.is_empty() {
        return true;
    }
    let Some(biome) = biome else {
        return false;
    };
    let b = biome[i] as f64;
    rule.biomes.contains(&b)
}

/// `wm && wm[i]===1`.
fn wetland_at(mask: Option<&[u8]>, i: usize) -> bool {
    mask.is_some_and(|m| m[i] == 1)
}

/// `sizeAt(r,x,y,k)`: `k==null` draws a fresh hash-derived fraction
/// (`hash(x,y,seed+2)`); an explicit `f` (relief mode's elevation-band
/// position) is used as-is.
fn size_at(rule: &ScatterRule, x: i32, y: i32, seed: i32, f: Option<f64>) -> f64 {
    let f = f.unwrap_or_else(|| hash(x, y, seed + 2));
    rule.min_size + (rule.max_size - rule.min_size) * f
}

/// `specificity` (reference line 7258): `(requireWetland?0:1)*1000 +
/// (biomes.length ? biomes.length : 9999)`. Lower sorts first = wins a
/// contested cell. See the module docs' v1.27 fix #1 for what this exists to
/// prevent.
fn specificity(rule: &ScatterRule) -> i64 {
    let wetland_term = if rule.require_wetland { 0 } else { 1000 };
    let biome_term = if rule.biomes.is_empty() {
        9999
    } else {
        rule.biomes.len() as i64
    };
    wetland_term + biome_term
}

/// `fits(x,y,space)`: true iff no already-placed relief icon in the
/// surrounding 3x3 buckets is within `space` cells (Euclidean, squared to
/// avoid a `sqrt`). A `Vec<Vec<(f64,f64)>>` bucket grid stands in for the
/// reference's linked-list buckets — occupant order inside a bucket never
/// affects the result, only how it's stored.
fn relief_fits(
    buckets: &[Vec<(f64, f64)>],
    bw: usize,
    bh: usize,
    cell: f64,
    x: f64,
    y: f64,
    space: f64,
) -> bool {
    let bx = (x / cell) as i64;
    let by = (y / cell) as i64;
    let s2 = space * space;
    for dy in -1..=1i64 {
        for dx in -1..=1i64 {
            let nx = bx + dx;
            let ny = by + dy;
            if nx < 0 || ny < 0 || nx as usize >= bw || ny as usize >= bh {
                continue;
            }
            for &(qx, qy) in &buckets[ny as usize * bw + nx as usize] {
                let ddx = qx - x;
                let ddy = qy - y;
                if ddx * ddx + ddy * ddy < s2 {
                    return false;
                }
            }
        }
    }
    true
}

/// `take(x,y)`: record a placed relief icon in its bucket.
fn relief_take(buckets: &mut [Vec<(f64, f64)>], bw: usize, cell: f64, x: f64, y: f64) {
    let bx = (x / cell) as usize;
    let by = (y / cell) as usize;
    buckets[by * bw + bx].push((x, y));
}

// ---------------------------------------------------------------------------
// iconSlotForItem, TREE_SLOT / SCATTER_SLOT
// ---------------------------------------------------------------------------

/// `TREE_SLOT[kind]||'tree_broadleaf'` (reference line 7289).
fn tree_slot(kind: Option<IconKind>) -> &'static str {
    match kind {
        Some(IconKind::Conifer) => "tree_conifer",
        Some(IconKind::Broadleaf) => "tree_broadleaf",
        Some(IconKind::Rainforest) => "tree_rainforest",
        Some(IconKind::Savanna) => "tree_savanna",
        Some(IconKind::Wetland) => "tree_wetland",
        _ => "tree_broadleaf",
    }
}

/// `SCATTER_SLOT[kind]||'shrub'` (reference line 7290).
fn scatter_slot(kind: Option<IconKind>) -> &'static str {
    match kind {
        Some(IconKind::Shrub) => "shrub",
        Some(IconKind::Cactus) => "cactus",
        Some(IconKind::Boulder) => "boulder",
        _ => "shrub",
    }
}

/// `iconSlotForItem` (reference line 7294): resolve a placed item to its
/// asset slot key, whether it carries an explicit `key` (the ruled path) or
/// only a legacy `cat`/`kind` pair.
///
/// `if(it.key) return it.key` is JS truthiness, not a presence check — an
/// empty-string key is falsy and falls through to the `cat`/`kind` branches,
/// reproduced here as `key.is_empty()`. See [`PlacedIcon::key`] for why that
/// case cannot arise from this crate's own placement engine.
pub fn icon_slot_for_item(item: &PlacedIcon) -> String {
    if let Some(key) = &item.key
        && !key.is_empty()
    {
        return key.clone();
    }
    match item.cat {
        IconCategory::Mountain => "mountain".to_string(),
        IconCategory::Hill => "hill".to_string(),
        IconCategory::Tree => tree_slot(item.kind).to_string(),
        IconCategory::Scatter | IconCategory::Ruled => scatter_slot(item.kind).to_string(),
    }
}

// ---------------------------------------------------------------------------
// spriteDrawRect
// ---------------------------------------------------------------------------

/// A sprite's destination rectangle on the map canvas — `dx`/`dy` are its
/// top-left corner, `dw`/`dh` its size.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpriteRect {
    pub dx: f64,
    pub dy: f64,
    pub dw: f64,
    pub dh: f64,
}

/// `spriteDrawRect(x,y,s,base,sw,sh)` (reference line 12173): bottom-centre
/// placement — the glyph's base sits on the map cell, like a label — scaled
/// to `base` and the sprite's own aspect ratio. `Math.max(1,sh)` guards a
/// zero-height source image from a division by zero.
pub fn sprite_draw_rect(x: f64, y: f64, s: f64, base: f64, sw: f64, sh: f64) -> SpriteRect {
    let dh = base * 2.2 * s;
    let dw = dh * (sw / sh.max(1.0));
    SpriteRect {
        dx: x - dw / 2.0,
        dy: y - dh,
        dw,
        dh,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scatter::ScatterMode;

    fn rule(over: impl FnOnce(&mut ScatterRule)) -> ScatterRule {
        let mut r = ScatterRule::default();
        over(&mut r);
        r
    }

    #[test]
    fn empty_grid_returns_no_items_instead_of_panicking() {
        let r = rule(|_| {});
        let rules = [("a", &r)];
        let opts = PlaceIconsRuledOpts::new(0, &rules);
        assert!(place_map_icons_ruled(&[], None, 0, 0, &opts).is_empty());
    }

    #[test]
    fn opts_new_matches_the_reference_default_t_gap() {
        let rules: [(&str, &ScatterRule); 0] = [];
        // Math.max(4, Math.round(W/110))
        assert_eq!(PlaceIconsRuledOpts::new(110, &rules).t_gap, 4);
        assert_eq!(PlaceIconsRuledOpts::new(1100, &rules).t_gap, 10);
        assert_eq!(PlaceIconsRuledOpts::new(1, &rules).t_gap, 4);
    }

    #[test]
    fn t_gap_of_zero_is_clamped_rather_than_hanging() {
        let r = rule(|r| r.mode = ScatterMode::Scatter);
        let rules = [("a", &r)];
        let mut opts = PlaceIconsRuledOpts::new(4, &rules);
        opts.t_gap = 0;
        let fld = vec![1.0; 16];
        // Must terminate at all -- a JS `for(;gy+=0)` would spin forever.
        let _ = place_map_icons_ruled(&fld, None, 4, 4, &opts);
    }

    #[test]
    fn biome_ok_rejects_a_non_integer_rule_biome_even_though_the_field_is_finite() {
        let r = rule(|r| r.biomes = vec![5.5]);
        let biome = [5u8];
        assert!(!biome_ok(&r, Some(&biome), 0));
        let r_int = rule(|r| r.biomes = vec![5.0]);
        assert!(biome_ok(&r_int, Some(&biome), 0));
    }

    #[test]
    fn biome_ok_any_land_when_biomes_empty_but_rejects_missing_biome_array() {
        let r = rule(|_| {});
        assert!(biome_ok(&r, None, 0));
        let r = rule(|r| r.biomes = vec![1.0]);
        assert!(!biome_ok(&r, None, 0));
    }

    #[test]
    fn icon_slot_for_item_prefers_key_over_cat() {
        let it = PlacedIcon {
            x: 0,
            y: 0,
            s: 1.0,
            key: Some("mountain_pack".to_string()),
            cat: IconCategory::Mountain,
            kind: None,
        };
        assert_eq!(icon_slot_for_item(&it), "mountain_pack");
    }

    #[test]
    fn icon_slot_for_item_treats_an_empty_key_as_absent() {
        let it = PlacedIcon {
            x: 0,
            y: 0,
            s: 1.0,
            key: Some(String::new()),
            cat: IconCategory::Ruled,
            kind: None,
        };
        // SCATTER_SLOT[undefined]||'shrub': 'ruled' matches none of the
        // named cat branches, so it falls to the final scatter fallback.
        assert_eq!(icon_slot_for_item(&it), "shrub");
    }

    #[test]
    fn icon_slot_for_item_legacy_tree_and_scatter_maps() {
        let tree = |kind| PlacedIcon {
            x: 0,
            y: 0,
            s: 1.0,
            key: None,
            cat: IconCategory::Tree,
            kind,
        };
        assert_eq!(icon_slot_for_item(&tree(Some(IconKind::Conifer))), "tree_conifer");
        assert_eq!(
            icon_slot_for_item(&tree(Some(IconKind::Broadleaf))),
            "tree_broadleaf"
        );
        assert_eq!(
            icon_slot_for_item(&tree(Some(IconKind::Rainforest))),
            "tree_rainforest"
        );
        assert_eq!(icon_slot_for_item(&tree(Some(IconKind::Savanna))), "tree_savanna");
        assert_eq!(icon_slot_for_item(&tree(Some(IconKind::Wetland))), "tree_wetland");
        // A scatter-only kind reaching the tree branch falls back, same as
        // TREE_SLOT[kind] being undefined.
        assert_eq!(icon_slot_for_item(&tree(Some(IconKind::Shrub))), "tree_broadleaf");
        assert_eq!(icon_slot_for_item(&tree(None)), "tree_broadleaf");

        let scatter = |kind| PlacedIcon {
            x: 0,
            y: 0,
            s: 1.0,
            key: None,
            cat: IconCategory::Scatter,
            kind,
        };
        assert_eq!(icon_slot_for_item(&scatter(Some(IconKind::Shrub))), "shrub");
        assert_eq!(icon_slot_for_item(&scatter(Some(IconKind::Cactus))), "cactus");
        assert_eq!(icon_slot_for_item(&scatter(Some(IconKind::Boulder))), "boulder");
        assert_eq!(icon_slot_for_item(&scatter(Some(IconKind::Conifer))), "shrub");
        assert_eq!(icon_slot_for_item(&scatter(None)), "shrub");
    }

    #[test]
    fn icon_slot_for_item_mountain_and_hill_cats() {
        let it = |cat| PlacedIcon {
            x: 0,
            y: 0,
            s: 1.0,
            key: None,
            cat,
            kind: None,
        };
        assert_eq!(icon_slot_for_item(&it(IconCategory::Mountain)), "mountain");
        assert_eq!(icon_slot_for_item(&it(IconCategory::Hill)), "hill");
    }

    #[test]
    fn specificity_orders_wetland_and_biome_specificity_as_the_reference_does() {
        let wetland_narrow = rule(|r| {
            r.require_wetland = true;
            r.biomes = vec![7.0];
        });
        let non_wetland_narrow = rule(|r| r.biomes = vec![7.0]);
        let wetland_any = rule(|r| r.require_wetland = true);
        let non_wetland_any = rule(|_| {});
        // Lower = wins first. Wetland-narrow < non-wetland-narrow (offset by
        // 1000) < wetland-any (9999) < non-wetland-any (10999, last resort).
        assert!(specificity(&wetland_narrow) < specificity(&non_wetland_narrow));
        assert!(specificity(&non_wetland_narrow) < specificity(&wetland_any));
        assert!(specificity(&wetland_any) < specificity(&non_wetland_any));
    }

    #[test]
    fn sprite_draw_rect_matches_reference_geometry() {
        let r = sprite_draw_rect(50.0, 80.0, 1.3, 4.5, 32.0, 48.0);
        assert!((r.dh - 12.870_000_000_000_001).abs() < 1e-9);
        assert!((r.dw - 8.58).abs() < 1e-9);
        assert!((r.dx - 45.71).abs() < 1e-9);
        assert!((r.dy - 67.13).abs() < 1e-9);
    }

    #[test]
    fn sprite_draw_rect_guards_a_zero_height_source() {
        let r = sprite_draw_rect(100.0, 200.0, 1.0, 5.0, 64.0, 0.0);
        assert_eq!((r.dw, r.dh), (704.0, 11.0));
    }
}
