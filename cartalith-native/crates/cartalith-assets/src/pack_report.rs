//! The pack report (`ASSET_PACK_TOOLS_PROPOSAL.md` AP-2, Ruling CQ): what is
//! wrong with a library before it is exported or applied, as a list of
//! findings a user can jump from to the slot concerned. **Read-only**: nothing
//! here changes the library, its pixels or the exported bytes.
//!
//! [`crate::library::run`]'s validator warnings come first, verbatim and in
//! their golden order, so the report is a superset of Validate, never a
//! rewording of it. The findings this module adds follow.
//!
//! What it deliberately does **not** report yet:
//!
//! - *(Since AP-6, 2026-10-19, near-duplicates are reported: flipped,
//!   tinted or shifted copies, [`crate::near_duplicate_groups`], one row per
//!   group after the exact duplicates. Until then only exact duplicates,
//!   [`crate::duplicate_groups`], the reference's hash, were named.)*
//! - "looks like line art on an opaque or keyed background": §9.3 adds it with
//!   AP-10, whose detector does not exist yet;
//! - colour conversions: the library model is pixel-free and never sees a
//!   colour chunk, so `cartalith-godot`'s session appends those findings from
//!   the reports it kept at import.
//!
//! Must never: guess a cause it did not measure, or encode "not measured" as
//! a number. A finding's `text` states what was measured and what follows.

use crate::library::{AssetDB, duplicate_groups, run};
use crate::near_dupes::near_duplicate_groups;
use crate::raster::{DecodedImage, render_item};
use crate::slicer::is_blank;
use crate::slots::Family;

/// How much a finding matters.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    /// Something that will look wrong, ship mislabelled, or export as nothing.
    Warn,
    /// True and worth knowing, but not a defect of the art (a family the map
    /// never draws, an empty family).
    Info,
}

impl Severity {
    /// The lowercase word the shell keys its dot colour on.
    pub fn key(self) -> &'static str {
        match self {
            Severity::Warn => "warn",
            Severity::Info => "info",
        }
    }
}

/// What kind of finding, so a shell can group or filter by it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FindingKind {
    /// One of `library::run`'s own warnings, verbatim.
    Validator,
    /// The pack licence is blank, so export writes "CC0".
    BlankLicence,
    /// A slot whose art the live map never draws.
    NotDrawn,
    /// An image with no pixel above the slicer's blank threshold.
    Blank,
    /// A sprite with no transparent pixel at all.
    Opaque,
    /// A source smaller than its family's bake size.
    Undersized,
    /// A source that is not square.
    NonSquare,
    /// A ground tile whose wrap edges differ more than its centre lines.
    Seam,
    /// An exact duplicate image (the reference's hash).
    Duplicate,
    /// A group of near-duplicate images: mirrored, slightly tinted or
    /// shifted copies (AP-6, [`crate::near_duplicate_groups`]).
    NearDuplicate,
    /// A colour conversion or colour warning at import (added by the session).
    Colour,
    /// A frozen family with no art at all.
    EmptyFamily,
}

impl FindingKind {
    /// The stable key the shell receives.
    pub fn key(self) -> &'static str {
        match self {
            FindingKind::Validator => "validator",
            FindingKind::BlankLicence => "licence",
            FindingKind::NotDrawn => "not_drawn",
            FindingKind::Blank => "blank",
            FindingKind::Opaque => "opaque",
            FindingKind::Undersized => "undersized",
            FindingKind::NonSquare => "non_square",
            FindingKind::Seam => "seam",
            FindingKind::Duplicate => "duplicate",
            FindingKind::NearDuplicate => "near_duplicate",
            FindingKind::Colour => "colour",
            FindingKind::EmptyFamily => "empty_family",
        }
    }
}

/// One row of the report.
#[derive(Debug, Clone, PartialEq)]
pub struct Finding {
    /// What kind.
    pub kind: FindingKind,
    /// How much it matters.
    pub severity: Severity,
    /// The slot it is about, so a row can jump to it; `None` for a
    /// pack-level finding.
    pub uid: Option<String>,
    /// The item (variant) index within the slot, when it is about one item.
    pub index: Option<usize>,
    /// Who it is about, in the user's words: `"Mountain #2"`, `"Sea marks"`,
    /// or `""` for the pack.
    pub label: String,
    /// What was found and what follows from it.
    pub text: String,
}

/// The seam measure of one ground tile, in 8-bit levels: the mean absolute
/// RGB difference between the pixels the wrap puts side by side (last column
/// against first, last row against first), and the same measure across the
/// tile's own centre lines as the baseline a seamless tile should match.
/// This is AP-4's own definition (`ASSET_PACK_TOOLS_PROPOSAL.md` §6).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SeamScore {
    /// Across the wrap edges.
    pub edge: f64,
    /// Across the centre lines.
    pub interior: f64,
}

/// A wrap edge counts as a seam above this multiple of the interior
/// baseline. AP-4's acceptance bar (≤ 1.25× after make-seamless), reused as
/// the detection threshold so the checker and the fixer agree; an
/// acceptance threshold, not a measurement.
pub const SEAM_RATIO: f64 = 1.25;

/// And only when the edge difference is also above this many levels, so a
/// near-flat tile whose interior baseline is ~0 is not flagged for noise.
/// A labelled judgement: two 8-bit levels is below a visible step on a
/// textured ground.
pub const SEAM_FLOOR: f64 = 2.0;

impl SeamScore {
    /// Whether this score is a seam by [`SEAM_RATIO`] and [`SEAM_FLOOR`].
    pub fn is_seam(&self) -> bool {
        self.edge > SEAM_FLOOR && self.edge > SEAM_RATIO * self.interior
    }
}

/// [`SeamScore`] of `img`. A tile under 2 px on a side has no centre line
/// and scores zero on both terms (never a seam).
pub fn seam_score(img: &DecodedImage) -> SeamScore {
    let (w, h) = (img.w as usize, img.h as usize);
    if w < 2 || h < 2 {
        return SeamScore { edge: 0.0, interior: 0.0 };
    }
    let px = |x: usize, y: usize| -> &[u8] { &img.rgba[(y * w + x) * 4..(y * w + x) * 4 + 3] };
    let diff = |a: &[u8], b: &[u8]| -> f64 {
        a.iter().zip(b).map(|(p, q)| f64::from(p.abs_diff(*q))).sum::<f64>() / 3.0
    };
    let mut edge = 0.0;
    let mut interior = 0.0;
    for y in 0..h {
        edge += diff(px(w - 1, y), px(0, y));
        interior += diff(px(w / 2 - 1, y), px(w / 2, y));
    }
    for x in 0..w {
        edge += diff(px(x, h - 1), px(x, 0));
        interior += diff(px(x, h / 2 - 1), px(x, h / 2));
    }
    let n = (w + h) as f64;
    SeamScore { edge: edge / n, interior: interior / n }
}

/// The slots the live map never draws, with the reason in words.
///
/// **Narrowed by AP-3 (Ruling CQ Q2, 2026-10-18).** Until AP-3 this also
/// named the settlement, POI, sea-mark and custom families, because
/// `LoadedPack` (`cartalith-godot`'s `pack.rs`) had no field for their art.
/// It now does (`LoadedPack::markers`), and the app's overlay draws a placed
/// icon of those families with its pack sprite (`map_overlay.gd`
/// `marker_art_enabled`, on in the app), so naming them would be false.
/// `textures.parchment` is still parsed and never decoded
/// (`LoadedPack::splat` doc), so it is the one slot left.
fn not_drawn_reason(family: Family, slot_id: &str) -> Option<&'static str> {
    match family {
        Family::Textures if slot_id == "parchment" => {
            Some("not drawn: the parchment texture is read from a pack but never decoded or used by the map")
        }
        _ => None,
    }
}

/// The user-facing name of a family, for an empty-family row.
fn family_title(family: Family) -> &'static str {
    match family {
        Family::Textures => "Splat channels",
        Family::Biomes => "Biome textures",
        Family::Terrains => "Terrain textures",
        Family::Icons => "Feature icons",
        Family::SeaMark => "Sea marks",
        Family::Settlement => "Settlement pins",
        Family::Trait => "Settlement traits",
        Family::Poi => "Points of interest",
        Family::Custom => "Custom icons",
    }
}

/// Build the report. `pixels(uid, index)` returns an item's decoded source
/// image (the session's parallel store); an item whose pixels it cannot
/// return is skipped for the pixel checks rather than guessed at.
pub fn pack_report<'a>(db: &AssetDB, pixels: impl Fn(&str, usize) -> Option<&'a DecodedImage>) -> Vec<Finding> {
    let mut out: Vec<Finding> = run(db)
        .into_iter()
        .map(|text| Finding {
            kind: FindingKind::Validator,
            severity: Severity::Warn,
            uid: None,
            index: None,
            label: String::new(),
            text,
        })
        .collect();
    if db.pack.license.trim().is_empty() {
        out.push(Finding {
            kind: FindingKind::BlankLicence,
            severity: Severity::Warn,
            uid: None,
            index: None,
            label: String::new(),
            text: "licence is blank: Export writes \"CC0\" into pack.json, which tells anyone who \
                   receives the pack the art is public domain"
                .to_string(),
        });
    }

    for uid in db.uids_in_order() {
        let items = db.items(&uid);
        if items.is_empty() {
            continue;
        }
        let Some(slot) = db.get(&uid) else { continue };
        let family = slot.family;
        let size = family.size();
        for (i, _item) in items.iter().enumerate() {
            let Some(img) = pixels(&uid, i) else { continue };
            let label = format!("{} #{}", slot.name, i + 1);
            let mut push = |kind, severity, text: String| {
                out.push(Finding { kind, severity, uid: Some(uid.clone()), index: Some(i), label: label.clone(), text });
            };
            if is_blank(img) {
                push(FindingKind::Blank, Severity::Warn, "blank: no pixel is visible (alpha above 8), so it exports as an empty image".to_string());
                continue;
            }
            if !family.opaque() && img.rgba.chunks_exact(4).all(|p| p[3] == 255) {
                push(
                    FindingKind::Opaque,
                    Severity::Warn,
                    format!("no transparent pixel: it draws as a solid {}×{} block, background included", img.w, img.h),
                );
            }
            if img.w.max(img.h) < size {
                push(
                    FindingKind::Undersized,
                    Severity::Warn,
                    format!("{}×{} source is smaller than the {size}² bake, so export enlarges it", img.w, img.h),
                );
            }
            if img.w != img.h {
                let (severity, text) = if family.opaque() {
                    (Severity::Warn, format!("{}×{} is not square: the {size}² tile is letterboxed onto black", img.w, img.h))
                } else {
                    (Severity::Info, format!("{}×{} is not square: the bake pads it with transparency", img.w, img.h))
                };
                push(FindingKind::NonSquare, severity, text);
            }
            if family.is_texture() {
                let baked = render_item(img, &items[i].transform, size, true);
                let s = seam_score(&baked);
                if s.is_seam() {
                    push(
                        FindingKind::Seam,
                        Severity::Warn,
                        format!(
                            "edges do not wrap: {:.1} levels across the wrap against {:.1} across the centre \
                             (over {SEAM_RATIO}×), so the tile repeats as a visible grid",
                            s.edge, s.interior
                        ),
                    );
                }
            }
        }
        if let Some(why) = not_drawn_reason(family, &slot.id) {
            out.push(Finding {
                kind: FindingKind::NotDrawn,
                severity: Severity::Info,
                uid: Some(uid.clone()),
                index: None,
                label: slot.name.clone(),
                text: why.to_string(),
            });
        }
    }

    for group in duplicate_groups(db) {
        let names: Vec<String> = group
            .iter()
            .map(|e| format!("{} #{}", db.get(&e.uid).map(|s| s.name.as_str()).unwrap_or(e.uid.as_str()), e.idx + 1))
            .collect();
        let first = &group[0];
        out.push(Finding {
            kind: FindingKind::Duplicate,
            severity: Severity::Warn,
            uid: Some(first.uid.clone()),
            index: Some(first.idx),
            label: names[0].clone(),
            text: format!("identical images: {}", names.join(" = ")),
        });
    }

    // AP-6: near-duplicate groups, one row per group, after the exact ones.
    // An Info, not a Warn: a mirrored or recoloured copy is often a variant on
    // purpose (AS-14 weights variants at render time), so the row says what
    // was measured and leaves the judgement to the user.
    for group in near_duplicate_groups(db, &pixels) {
        let names: Vec<String> = group
            .iter()
            .map(|e| format!("{} #{}", db.get(&e.uid).map(|s| s.name.as_str()).unwrap_or(e.uid.as_str()), e.idx + 1))
            .collect();
        let first = &group[0];
        out.push(Finding {
            kind: FindingKind::NearDuplicate,
            severity: Severity::Info,
            uid: Some(first.uid.clone()),
            index: Some(first.idx),
            label: names[0].clone(),
            text: format!(
                "near-duplicates (the same drawing mirrored, slightly tinted or shifted): {}",
                names.join(" ~ ")
            ),
        });
    }

    if db.total_items() > 0 {
        for family in Family::ALL {
            if family == Family::Custom || db.filled_count(family) > 0 {
                continue;
            }
            out.push(Finding {
                kind: FindingKind::EmptyFamily,
                severity: Severity::Info,
                uid: None,
                index: None,
                label: family_title(family).to_string(),
                text: "no art in this family: the map keeps its own look for every slot".to_string(),
            });
        }
    }
    out
}

/// Each finding kind against a fixture that fires it and one that does not.
#[cfg(test)]
mod tests {
    use super::*;
    use crate::library::LibraryItem;
    use crate::raster::item_hash;
    use std::collections::HashMap;

    /// A `w`×`h` image whose pixel at (x, y) is `f(x, y)`.
    fn img(w: u32, h: u32, f: impl Fn(u32, u32) -> [u8; 4]) -> DecodedImage {
        let mut v = Vec::with_capacity((w * h * 4) as usize);
        for y in 0..h {
            for x in 0..w {
                v.extend_from_slice(&f(x, y));
            }
        }
        DecodedImage::new(w, h, v).unwrap()
    }

    /// A library plus its pixel store, filled by `add`.
    struct Lib {
        db: AssetDB,
        px: HashMap<String, Vec<DecodedImage>>,
    }

    impl Lib {
        fn new() -> Self {
            let mut db = AssetDB::new();
            db.pack.name = "P".into();
            db.pack.license = "CC-BY-4.0".into();
            Lib { db, px: HashMap::new() }
        }
        fn add(&mut self, uid: &str, im: DecodedImage) {
            assert!(self.db.add_item(uid, LibraryItem::new("x", item_hash(&im))), "{uid}");
            self.px.entry(uid.to_string()).or_default().push(im);
        }
        fn report(&self) -> Vec<Finding> {
            pack_report(&self.db, |u, i| self.px.get(u).and_then(|v| v.get(i)))
        }
        fn kinds(&self, uid: &str) -> Vec<FindingKind> {
            self.report().into_iter().filter(|f| f.uid.as_deref() == Some(uid)).map(|f| f.kind).collect()
        }
    }

    /// A clean 256² sprite: opaque disc on transparency, unique by `seed`.
    fn good_sprite(seed: u8) -> DecodedImage {
        img(256, 256, |x, y| {
            let (dx, dy) = (x as i32 - 128, y as i32 - 128);
            if dx * dx + dy * dy < 100 * 100 { [seed, 40, 90, 255] } else { [0, 0, 0, 0] }
        })
    }

    /// A seamless 512² tile: a pattern periodic in the tile, unique by `seed`.
    fn seamless_tile(seed: u8) -> DecodedImage {
        img(512, 512, |x, y| [seed, ((x / 4 + y / 4) % 2 * 40 + 60) as u8, 50, 255])
    }

    /// Protects: a clean sprite and a clean seamless tile raise no per-item
    /// finding at all (the "does not fire" half of every pixel check).
    #[test]
    fn clean_art_raises_no_item_finding() {
        let mut l = Lib::new();
        l.add("icons:hill", good_sprite(1));
        l.add("textures:grass", seamless_tile(2));
        assert_eq!(l.kinds("icons:hill"), vec![]);
        assert_eq!(l.kinds("textures:grass"), vec![]);
    }

    /// Protects: blank, opaque, undersized and non-square each fire on a
    /// fixture built to trip exactly that check, and a blank image reports
    /// only blank (the other checks are meaningless on nothing).
    #[test]
    fn each_sprite_check_fires_on_its_own_fixture() {
        let mut l = Lib::new();
        l.add("icons:hill", img(256, 256, |_, _| [0, 0, 0, 5]));
        l.add("icons:boulder", img(256, 256, |_, _| [9, 9, 9, 255]));
        l.add("icons:shrub", img(64, 64, |x, _| if x < 32 { [1, 2, 3, 255] } else { [0, 0, 0, 0] }));
        l.add("icons:cactus", img(256, 128, |x, _| if x < 100 { [1, 2, 3, 255] } else { [0, 0, 0, 0] }));
        l.add("textures:rock", img(512, 256, |x, y| [(x % 7) as u8, (y % 5) as u8, 3, 255]));
        assert_eq!(l.kinds("icons:hill"), vec![FindingKind::Blank]);
        assert_eq!(l.kinds("icons:boulder"), vec![FindingKind::Opaque]);
        assert_eq!(l.kinds("icons:shrub"), vec![FindingKind::Undersized]);
        assert_eq!(l.kinds("icons:cactus"), vec![FindingKind::NonSquare]);
        let rock: Vec<Finding> = l.report().into_iter().filter(|f| f.uid.as_deref() == Some("textures:rock")).collect();
        assert!(rock.iter().any(|f| f.kind == FindingKind::NonSquare && f.severity == Severity::Warn), "{rock:?}");
        let cactus: Vec<Finding> = l.report().into_iter().filter(|f| f.uid.as_deref() == Some("icons:cactus")).collect();
        assert_eq!(cactus[0].severity, Severity::Info, "a sprite is padded, not letterboxed");
        assert_eq!(cactus[0].label, "Cactus #1");
    }

    /// Protects: the seam check fires on a left-to-right ramp (wrap edge
    /// ~255 levels, centre ~0.5) and not on a periodic tile, and the score
    /// is AP-4's measure: literal values on a known ramp.
    #[test]
    fn the_seam_check_fires_on_a_ramp_and_not_on_a_periodic_tile() {
        // A one-level-per-pixel ramp, so a measure taken one column off the
        // centre line reads 2 levels, not 1, and is caught.
        let ramp = img(256, 256, |x, _| [x as u8, 0, 0, 255]);
        let s = seam_score(&ramp);
        // Columns: |255 - 0| / 3 per row; rows: identical, 0. Interior
        // columns 127 and 128 -> |1| / 3. Each averaged over 256 + 256.
        assert!((s.edge - (255.0 / 3.0) * 256.0 / 512.0).abs() < 1e-9, "{s:?}");
        assert!((s.interior - (1.0 / 3.0) * 256.0 / 512.0).abs() < 1e-9, "{s:?}");
        assert!(s.is_seam());
        assert!(!seam_score(&seamless_tile(3)).is_seam());
        let mut l = Lib::new();
        l.add("textures:sand", img(512, 512, |x, _| [(x / 2) as u8, 0, 0, 255]));
        assert_eq!(l.kinds("textures:sand"), vec![FindingKind::Seam]);
    }

    /// Protects: the seam thresholds are the documented literals: a ratio
    /// just over 1.25 above the floor fires, just under does not, and an edge
    /// at the floor never fires however small the interior.
    #[test]
    fn seam_thresholds_are_the_documented_literals() {
        assert!(SeamScore { edge: 12.6, interior: 10.0 }.is_seam());
        assert!(!SeamScore { edge: 12.4, interior: 10.0 }.is_seam());
        assert!(!SeamScore { edge: 2.0, interior: 0.0 }.is_seam());
        assert!(SeamScore { edge: 2.01, interior: 0.0 }.is_seam());
    }

    /// Protects: a blank licence fires and a set one does not; validator
    /// warnings come first, verbatim; not-drawn fires for the parchment
    /// texture and, since AP-3 draws them, not for settlement, POI, sea marks,
    /// custom or icons; an
    /// exact duplicate names both items; an empty family is listed only when
    /// the library holds something.
    #[test]
    fn pack_level_and_family_findings() {
        let mut l = Lib::new();
        assert!(l.report().iter().all(|f| f.kind == FindingKind::Validator), "an empty library: only Validate's own warning");
        l.add("icons:hill", good_sprite(1));
        l.add("icons:mountain", good_sprite(1));
        l.add("settlement:city", good_sprite(2));
        l.add("poi:ruin", good_sprite(3));
        l.add("seamarks:buoy", good_sprite(4));
        l.add("textures:parchment", seamless_tile(5));
        let cuid = l.db.add_custom_slot("Mill", Some("Farm")).uid.clone();
        l.add(&cuid, good_sprite(6));
        let r = l.report();
        assert_eq!(r[0].kind, FindingKind::Validator);
        assert_eq!(r[0].text, "Identical images: Mountain#1 = Hill#1", "verbatim from library::run");
        assert!(!r.iter().any(|f| f.kind == FindingKind::BlankLicence));
        let not_drawn: Vec<&str> = r.iter().filter(|f| f.kind == FindingKind::NotDrawn).map(|f| f.uid.as_deref().unwrap()).collect();
        // AP-3 (2026-10-18). Old -> new: ["textures:parchment",
        // "seamarks:buoy", "settlement:city", "poi:ruin", <custom uid>] ->
        // ["textures:parchment"]: the four marker families are drawn now.
        assert_eq!(not_drawn, vec!["textures:parchment"]);
        let dup: Vec<&Finding> = r.iter().filter(|f| f.kind == FindingKind::Duplicate).collect();
        assert_eq!(dup.len(), 1);
        assert_eq!(dup[0].text, "identical images: Mountain #1 = Hill #1");
        let empty: Vec<&str> = r.iter().filter(|f| f.kind == FindingKind::EmptyFamily).map(|f| f.label.as_str()).collect();
        assert_eq!(empty, vec!["Biome textures", "Terrain textures", "Settlement traits"]);
        l.db.pack.license = "  ".into();
        let lic: Vec<Finding> = l.report().into_iter().filter(|f| f.kind == FindingKind::BlankLicence).collect();
        assert_eq!(lic.len(), 1);
        assert!(lic[0].text.contains("CC0"));
    }

    /// A 256² sprite with real structure: a disc whose shade ramps left to
    /// right, so it has a direction a mirror reverses.
    fn ramp_disc(r: u8) -> DecodedImage {
        img(256, 256, |x, y| {
            let (dx, dy) = (x as i32 - 128, y as i32 - 128);
            if dx * dx + dy * dy < 100 * 100 { [r, (40 + x / 2) as u8, 90, 255] } else { [0, 0, 0, 0] }
        })
    }

    /// Protects: AP-6 in the report. A sprite and its mirror in two slots
    /// give exactly one near-duplicate row (Info, naming both, jumping to the
    /// first), after the exact-duplicate rows; a pair of exact copies alone
    /// gives an exact row and no near row; a clearly different colour of the
    /// same drawing gives none.
    #[test]
    fn near_duplicates_are_one_row_per_group_after_exact_ones() {
        let mut l = Lib::new();
        let a = ramp_disc(30);
        let mut mirror = a.clone();
        for y in 0..256usize {
            for x in 0..256usize {
                let (s, d) = ((y * 256 + x) * 4, (y * 256 + 255 - x) * 4);
                mirror.rgba[d..d + 4].copy_from_slice(&a.rgba[s..s + 4]);
            }
        }
        l.add("icons:hill", a.clone());
        l.add("icons:mountain", mirror);
        l.add("icons:shrub", ramp_disc(220));
        l.add("icons:shrub", ramp_disc(220));
        let r = l.report();
        let near: Vec<&Finding> = r.iter().filter(|f| f.kind == FindingKind::NearDuplicate).collect();
        assert_eq!(near.len(), 1, "{near:?}");
        assert_eq!(near[0].severity, Severity::Info);
        assert_eq!(near[0].kind.key(), "near_duplicate");
        let first_uid = near[0].uid.clone().unwrap();
        assert!(first_uid == "icons:hill" || first_uid == "icons:mountain");
        assert!(near[0].text.contains("Hill #1") && near[0].text.contains("Mountain #1"), "{}", near[0].text);
        assert!(!near[0].text.contains("Shrub"), "the exact pair and the other colour are not in it");
        let dup_at = r.iter().position(|f| f.kind == FindingKind::Duplicate).unwrap();
        let near_at = r.iter().position(|f| f.kind == FindingKind::NearDuplicate).unwrap();
        assert!(dup_at < near_at, "exact duplicates come first");
    }
}
