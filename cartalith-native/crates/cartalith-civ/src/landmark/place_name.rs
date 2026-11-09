//! Ruling EI (owner, 2026-11-08): the name a placed landmark is linked by.
//!
//! A landmark has an id (a list position), a kind, a class and a cell, and **no name**
//! (`Landmark`'s own doc), so a vault link `[[...]]` had nothing to write between its
//! brackets. The owner chose a *generated* name: the landmark's type label plus its
//! nearest named place, for example `Waterfall near Vhal Serai`.
//!
//! **Why this lives here and not in the Godot bridge.** The name is a pure function of
//! the landmarks and the world's places, it can be unit-tested without a Godot runtime
//! (`cartalith-godot`'s `GodotClass` code cannot be, `landmark_dict_tests` says why), and
//! it changes **no generated output**: nothing in the landmark pass, a save or a golden
//! reads it. Because of that the tests below are golden-free; they pin behaviour, not a
//! reference value.
//!
//! **What the name must never do.**
//! * Change because a *different* landmark was placed elsewhere. A name is derived from
//!   the landmark's own kind and cell and from the world's places only. The single
//!   exception is the tie rule, which by its nature looks at landmarks that would share
//!   the name (same type, same nearest place) and nobody else.
//! * Invent a place. With no named place near enough, the name falls back to the type
//!   label and the cell, `Waterfall (120, 64)` -- the same designation
//!   `vault_landmark_entity_id` already used for a note's suggested path -- and the
//!   result says it was a fallback ([`NameSource::Fallback`]), so a caller never has to
//!   guess from the text.
//! * Overwrite a name the user gave. [`landmark_names`]'s `user_names` wins outright and
//!   is never given a suffix.
//!
//! The input order of `landmarks` never changes any name (a test permutes it).

use super::{kind_spec, Landmark};
use std::collections::BTreeMap;

/// How near "near" is, as a fraction of the longer map side, measured in grid cells.
///
/// **A labelled judgement, not a measurement.** 0.15 keeps "near" to a place a traveller
/// could plausibly name the spot by (about 300 cells on a 2048-wide map), and stops a
/// landmark in an empty quarter of the map being called "near" a town hundreds of cells
/// away. A landmark beyond it takes the fallback. Changing it renames only landmarks whose
/// nearest place lies in the changed band, and changes no generated output.
pub const NEAR_FRACTION: f64 = 0.15;

/// A named place a landmark can be said to be near: a settlement's name and its cell.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NamedPlace<'a> {
    pub name: &'a str,
    pub x: f64,
    pub y: f64,
}

/// Where a landmark's name came from, so a caller can say it instead of parsing the text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NameSource {
    /// The user gave the landmark this name; it replaced the generated one.
    UserGiven,
    /// Type plus nearest named place (with the tie suffix if one was needed).
    NearPlace,
    /// No named place was near enough, or the world has none: type plus cell.
    Fallback,
}

/// One landmark's name. `key` is `Landmark::key()`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LandmarkName {
    pub key: String,
    pub name: String,
    pub source: NameSource,
}

/// The eight-point compass word for the direction from a place to a landmark. `y` grows
/// southward on the map, so a landmark with a smaller `y` is north of the place.
///
/// A landmark on the place's own cell has no direction; that is said as `same cell`
/// rather than given an arbitrary one.
fn bearing(dx: f64, dy: f64) -> &'static str {
    if dx == 0.0 && dy == 0.0 {
        return "same cell";
    }
    const WORDS: [&str; 8] = [
        "north", "north-east", "east", "south-east", "south", "south-west", "west", "north-west",
    ];
    // Degrees clockwise from north, in [0, 360).
    let deg = dx.atan2(-dy).to_degrees().rem_euclid(360.0);
    WORDS[(((deg + 22.5) / 45.0) as usize) % 8]
}

/// The generated name of every landmark in `landmarks`, in the same order.
///
/// * `places` -- the world's named places (settlements). A blank name is skipped: it
///   would read `Waterfall near ` and a made-up stand-in is worse.
/// * `map_w`, `map_h` -- the grid's size in cells, which scales [`NEAR_FRACTION`].
/// * `user_names` -- `Landmark::key()` to a name the user gave; a blank value is ignored.
///
/// **Nearest place.** Euclidean distance in cells; a tie in distance goes to the place
/// whose name sorts first, then to the lower `x`, then `y`, so the answer never depends
/// on the order `places` came in.
///
/// **Ties between two generated names.** When two landmarks would share one name (same
/// type, same nearest place; compared case-insensitively) every one of them gets the
/// compass word of its direction from that place, `Waterfall near Vhal Serai (north-east)`,
/// which depends on the landmark and the place alone, not on which was found first. If
/// two still share a name (same direction) the cell is added as well,
/// `(north-east, 120, 64)`; a cell is unique to a landmark of one type, so that ends it.
/// A landmark whose name is unique takes no suffix, which means a twin placed later at the
/// same place adds a suffix to the earlier one: the only way a name moves, and it is the
/// tie the owner asked to have told apart.
pub fn landmark_names(
    landmarks: &[Landmark],
    places: &[NamedPlace<'_>],
    map_w: usize,
    map_h: usize,
    user_names: &BTreeMap<String, String>,
) -> Vec<LandmarkName> {
    let near = NEAR_FRACTION * map_w.max(map_h) as f64;
    let named: Vec<&NamedPlace<'_>> = places.iter().filter(|p| !p.name.trim().is_empty()).collect();

    // Pass 1: the base name, and (for a near-place name) the facts the tie rule needs.
    struct Base {
        key: String,
        base: String,
        source: NameSource,
        bearing: &'static str,
        cell: (usize, usize),
    }
    let bases: Vec<Base> = landmarks
        .iter()
        .map(|lm| {
            let key = lm.key();
            // A kind the registry does not know keeps its key as written: a machine token is
            // honest where a guessed label is not.
            let label = kind_spec(&lm.kind).map_or(lm.kind.as_str(), |s| s.label);
            if let Some(n) = user_names.get(&key).map(|n| n.trim()).filter(|n| !n.is_empty()) {
                return Base { key, base: n.to_string(), source: NameSource::UserGiven, bearing: "", cell: (lm.x, lm.y) };
            }
            let (x, y) = (lm.x as f64, lm.y as f64);
            let nearest = named
                .iter()
                .map(|p| (((p.x - x).powi(2) + (p.y - y).powi(2)).sqrt(), *p))
                .min_by(|a, b| {
                    a.0.total_cmp(&b.0)
                        .then_with(|| a.1.name.cmp(b.1.name))
                        .then_with(|| a.1.x.total_cmp(&b.1.x))
                        .then_with(|| a.1.y.total_cmp(&b.1.y))
                });
            match nearest {
                Some((d, p)) if d <= near => Base {
                    key,
                    base: format!("{label} near {}", p.name.trim()),
                    source: NameSource::NearPlace,
                    bearing: bearing(x - p.x, y - p.y),
                    cell: (lm.x, lm.y),
                },
                _ => Base {
                    key,
                    base: format!("{label} ({}, {})", lm.x, lm.y),
                    source: NameSource::Fallback,
                    bearing: "",
                    cell: (lm.x, lm.y),
                },
            }
        })
        .collect();

    // Pass 2: who shares a name. A user-given name counts as occupying its text so a
    // generated one that equals it is the one that moves.
    fn count(names: impl Iterator<Item = String>) -> BTreeMap<String, usize> {
        let mut m: BTreeMap<String, usize> = BTreeMap::new();
        for n in names {
            *m.entry(n.to_lowercase()).or_default() += 1;
        }
        m
    }
    let first = count(bases.iter().map(|b| b.base.clone()));
    let with_bearing = |b: &Base| {
        if b.source == NameSource::NearPlace && first[&b.base.to_lowercase()] > 1 {
            format!("{} ({})", b.base, b.bearing)
        } else {
            b.base.clone()
        }
    };
    let second = count(bases.iter().map(with_bearing));
    bases
        .iter()
        .map(|b| {
            let mut name = with_bearing(b);
            if b.source == NameSource::NearPlace && second[&name.to_lowercase()] > 1 {
                name = format!("{} ({}, {}, {})", b.base, b.bearing, b.cell.0, b.cell.1);
            }
            LandmarkName { key: b.key.clone(), name, source: b.source }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::landmark::LandmarkClass;
    use std::collections::BTreeSet;

    fn lm(kind: &str, x: usize, y: usize) -> Landmark {
        Landmark {
            id: 1,
            kind: kind.to_string(),
            class: LandmarkClass::Local,
            x,
            y,
            elevation: 0.0,
            score: 0.5,
            importance: 0.5,
            causal: Vec::new(),
            seed: 7,
        }
    }

    fn place(name: &str, x: f64, y: f64) -> NamedPlace<'_> {
        NamedPlace { name, x, y }
    }

    fn names_of(ls: &[Landmark], ps: &[NamedPlace<'_>]) -> BTreeMap<String, LandmarkName> {
        landmark_names(ls, ps, 1000, 500, &BTreeMap::new()).into_iter().map(|n| (n.key.clone(), n)).collect()
    }

    /// Protects: the Ruling EI format, `<type label> near <nearest named place>`, using the
    /// registry's label (not the key) and the *nearest* place, not the first listed.
    #[test]
    fn a_landmark_is_named_by_its_type_and_its_nearest_place() {
        let ps = [place("Far Town", 400.0, 400.0), place("Vhal Serai", 110.0, 70.0)];
        let l = lm("waterfall", 120, 64);
        let n = names_of(&[l.clone()], &ps);
        let got = &n[&l.key()];
        assert_eq!(got.name, "Waterfall near Vhal Serai");
        assert_eq!(got.source, NameSource::NearPlace);
    }

    /// Protects: stability. Adding landmarks elsewhere (another kind, another place),
    /// reordering the list and reordering the places leave an existing name unchanged.
    #[test]
    fn a_name_survives_other_landmarks_and_any_input_order() {
        let ps = [place("Vhal Serai", 110.0, 70.0), place("Kelvhold", 800.0, 300.0)];
        let a = lm("waterfall", 120, 64);
        let before = names_of(&[a.clone()], &ps)[&a.key()].name.clone();
        let more = vec![lm("peak", 790, 310), lm("spring", 805, 295), a.clone(), lm("waterfall", 810, 300)];
        let after = names_of(&more, &ps)[&a.key()].name.clone();
        assert_eq!(before, after);
        let mut rev = more.clone();
        rev.reverse();
        let mut ps_rev = ps;
        ps_rev.reverse();
        let r = names_of(&rev, &ps_rev);
        let f = names_of(&more, &ps);
        assert_eq!(r, f, "input order must not change any name");
    }

    /// Protects: a user-given name replaces the generated one, is trimmed, is never
    /// suffixed, and a blank user name is ignored rather than naming the landmark "".
    #[test]
    fn a_user_given_name_overrides_and_a_blank_one_does_not() {
        let ps = [place("Vhal Serai", 110.0, 70.0)];
        let (a, b) = (lm("waterfall", 120, 64), lm("waterfall", 100, 80));
        let mut over = BTreeMap::new();
        over.insert(a.key(), "  The Weeping Stair ".to_string());
        over.insert(b.key(), "   ".to_string());
        let out = landmark_names(&[a.clone(), b.clone()], &ps, 1000, 500, &over);
        assert_eq!(out[0].name, "The Weeping Stair");
        assert_eq!(out[0].source, NameSource::UserGiven);
        assert_eq!(out[1].source, NameSource::NearPlace);
        // b is alone in its group now (a's generated name is gone), so it takes no suffix.
        assert_eq!(out[1].name, "Waterfall near Vhal Serai");
    }

    /// Protects: the tie rule. Two same-type landmarks near one place both get their
    /// compass word, which is the same whatever the order; a third in the same direction
    /// also gets its cell; a unique name takes no suffix.
    #[test]
    fn ties_are_told_apart_by_direction_then_cell() {
        let ps = [place("Vhal Serai", 100.0, 100.0)];
        let ne = lm("waterfall", 120, 80);
        let sw = lm("waterfall", 80, 120);
        let ne2 = lm("waterfall", 130, 70);
        let out = names_of(&[ne.clone(), sw.clone()], &ps);
        assert_eq!(out[&ne.key()].name, "Waterfall near Vhal Serai (north-east)");
        assert_eq!(out[&sw.key()].name, "Waterfall near Vhal Serai (south-west)");
        let out3 = names_of(&[ne.clone(), sw.clone(), ne2.clone()], &ps);
        assert_eq!(out3[&ne.key()].name, "Waterfall near Vhal Serai (north-east, 120, 80)");
        assert_eq!(out3[&ne2.key()].name, "Waterfall near Vhal Serai (north-east, 130, 70)");
        assert_eq!(out3[&sw.key()].name, "Waterfall near Vhal Serai (south-west)");
        let names: BTreeSet<_> = out3.values().map(|n| n.name.to_lowercase()).collect();
        assert_eq!(names.len(), 3, "every name is distinct");
        // A different kind at the same place is not a tie.
        let spring = lm("spring", 120, 80);
        let mixed = names_of(&[ne.clone(), spring.clone()], &ps);
        assert_eq!(mixed[&ne.key()].name, "Waterfall near Vhal Serai");
        assert_eq!(mixed[&spring.key()].name, "Spring near Vhal Serai");
    }

    /// Protects: the eight compass words and the north-is-up convention (a smaller `y` is
    /// north), plus the on-the-cell case, which has no direction.
    #[test]
    fn bearing_words_follow_the_map_with_north_up() {
        assert_eq!(bearing(0.0, -5.0), "north");
        assert_eq!(bearing(5.0, -5.0), "north-east");
        assert_eq!(bearing(5.0, 0.0), "east");
        assert_eq!(bearing(5.0, 5.0), "south-east");
        assert_eq!(bearing(0.0, 5.0), "south");
        assert_eq!(bearing(-5.0, 5.0), "south-west");
        assert_eq!(bearing(-5.0, 0.0), "west");
        assert_eq!(bearing(-5.0, -5.0), "north-west");
        assert_eq!(bearing(0.0, 0.0), "same cell");
        // Off the axes: a sector is 45 degrees wide centred on its word, so 18.4 degrees off north is
        // still north and 26.6 is north-east (MUTATION: dropping the 22.5 half-sector offset flips both).
        assert_eq!(bearing(1.0, -3.0), "north");
        assert_eq!(bearing(1.0, -2.0), "north-east");
    }

    /// Protects: "no nearby place" is never an invented place. A landmark beyond
    /// `NEAR_FRACTION`, a world with no places, and a place with a blank name all give the
    /// stated fallback (type and cell) and say `Fallback`.
    #[test]
    fn no_nearby_place_falls_back_to_type_and_cell() {
        let l = lm("waterfall", 120, 64);
        // map 1000 wide -> near is 150 cells; this town is 400 away.
        let far = [place("Far Town", 520.0, 64.0)];
        for ps in [&far[..], &[][..], &[place("  ", 121.0, 64.0)][..]] {
            let got = landmark_names(&[l.clone()], ps, 1000, 500, &BTreeMap::new());
            assert_eq!(got[0].name, "Waterfall (120, 64)");
            assert_eq!(got[0].source, NameSource::Fallback);
            assert!(!got[0].name.contains("near"));
        }
    }

    /// Protects: the boundary of "near" (a place exactly `NEAR_FRACTION` away still counts,
    /// one cell further does not) and the nearest-place tie-break on equal distance, which
    /// is by name, not by list order.
    #[test]
    fn the_near_boundary_and_the_equal_distance_tie_break() {
        let l = lm("peak", 500, 250);
        let edge = [place("Edge", 500.0 - 150.0, 250.0)];
        assert_eq!(landmark_names(&[l.clone()], &edge, 1000, 500, &BTreeMap::new())[0].source, NameSource::NearPlace);
        let past = [place("Past", 500.0 - 151.0, 250.0)];
        assert_eq!(landmark_names(&[l.clone()], &past, 1000, 500, &BTreeMap::new())[0].source, NameSource::Fallback);
        let tie = [place("Zed", 510.0, 250.0), place("Alder", 490.0, 250.0)];
        assert_eq!(landmark_names(&[l.clone()], &tie, 1000, 500, &BTreeMap::new())[0].name, "Peak near Alder");
        let tie_rev = [tie[1], tie[0]];
        assert_eq!(landmark_names(&[l], &tie_rev, 1000, 500, &BTreeMap::new())[0].name, "Peak near Alder");
    }

    /// Protects: every registered kind has a distinct, non-empty label, which the tie rule
    /// and the fallback (`label (x, y)`) both lean on to be unique.
    #[test]
    fn every_kind_label_is_non_empty_and_distinct() {
        let mut seen = BTreeSet::new();
        for k in super::super::kinds() {
            assert!(!k.label.trim().is_empty(), "{} has no label", k.key);
            assert!(seen.insert(k.label.to_lowercase()), "label {} repeats", k.label);
        }
    }
}
