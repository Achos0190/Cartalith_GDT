//! LLM-5 (`LLM_ASSISTANT_SCOPE.md` s.3, s.5b.4): the two read-only engine
//! answers the shell's grouped undo (`shell/undo_group.gd`) needs and could
//! not get before.
//!
//! # What it is
//!
//! The grouped undo itself is GDScript, because three of its five member
//! kinds keep their history in the shell (`look_history.gd`, the colour-paint
//! tool's own stack, and the new parameter member, which has no history
//! anywhere). The height and way steps keep theirs here, in `WorldGen`'s
//! `HeightUndo` stack, way store and `HistoryLedger`. This file adds two
//! `#[func]`s over that state and changes none of it:
//!
//! - [`WorldGen::undo_commit_seq`]: the ledger's monotonic commit counter
//!   (`HistoryLedger::last_seq`). It moves on every recorded commit and on a
//!   redo (which records a fresh row) and never on an undo, which is what lets
//!   the group tell "nothing was committed since I was undone" (so the engine's
//!   redo tail is still mine) from "something was". Before this, the shell
//!   could see row seqs but not the counter: a removed row hides how many
//!   commits happened.
//! - [`WorldGen::redo_offer_ids`]: the identities of what `redo_last()`
//!   would re-apply, in the order it would, beside the labels `redo_labels()`
//!   already gives. An undone group checks that the engine's next redo offers
//!   are still its own steps (`undo::next_step_id` says why labels cannot).
//! - [`WorldGen::edit_state_digest`]: FNV-1a hashes of the editable world
//!   state an undo group can touch (height field, geology column, hand-drawn
//!   ways, parameters, colour paint) plus the cheap identities of the two
//!   generated way stores. LLM-5's bar is that the world after undoing a group
//!   *hashes equal* to the world before it; this is the hash.
//!
//! # What it must never do
//!
//! - Write anything. Both functions take `&self`.
//! - Report a part that does not exist as a plausible value: with no world
//!   every key is omitted (`{}`), and with no geology column the `column` key
//!   is omitted rather than hashed as empty (`MISTAKES.md`, absent is not a
//!   value).
//! - Be read as a full fingerprint of the generated way stores: those two keys
//!   are `context_pick_bridge::fp_generated`/`fp_sea`, which hash ends, counts,
//!   lengths and names but not every vertex (their own doc says why). No group
//!   member edits a generated way's vertices, so that identity is enough here;
//!   the hand-drawn store, which a way drag does edit, is hashed vertex by
//!   vertex.

use godot::prelude::*;

use cartalith_civ::tools::ManualWay;

use crate::{context_pick_bridge, params, WorldGen, WorldSource};

/// FNV-1a 64-bit offset basis, from the published definition (Fowler, Noll,
/// Vo; the same pair `WorldGen::drawn_water_digest` and
/// `tests/color_space.rs::fnv1a` use).
const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
/// FNV-1a 64-bit prime, same source as [`FNV_OFFSET`].
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

/// FNV-1a 64 over `bytes`, continuing from `h` (start with [`FNV_OFFSET`]).
/// Order-sensitive, which is the point: a digest must move when two cells
/// swap values.
fn fnv1a_from(h: u64, bytes: impl IntoIterator<Item = u8>) -> u64 {
    bytes.into_iter().fold(h, |h, b| (h ^ b as u64).wrapping_mul(FNV_PRIME))
}

/// FNV-1a 64 of a byte string from the offset basis.
pub fn fnv1a(bytes: &[u8]) -> u64 {
    fnv1a_from(FNV_OFFSET, bytes.iter().copied())
}

/// Digest of an `f32` field by its **bit patterns**, little-endian, length
/// first. Bits rather than values so `-0.0` and `0.0` (and two NaN payloads)
/// count as different states: an undo that writes back a different bit
/// pattern has not restored the field.
pub fn field_digest(field: &[f32]) -> u64 {
    let h = fnv1a_from(FNV_OFFSET, (field.len() as u64).to_le_bytes());
    fnv1a_from(h, field.iter().flat_map(|v| v.to_bits().to_le_bytes()))
}

/// Digest of the hand-drawn way store, every vertex included (unlike
/// `context_pick_bridge::fp_manual`, which is a cheap identity). `None` (no
/// store yet) and `Some(&[])` (an emptied store) hash differently, because
/// they are different states and an undo must not turn one into the other.
pub fn manual_ways_digest(ways: Option<&[ManualWay]>) -> u64 {
    let Some(ws) = ways else { return fnv1a(b"no-store") };
    let mut h = fnv1a_from(FNV_OFFSET, (ws.len() as u64).to_le_bytes());
    for w in ws {
        h = fnv1a_from(h, (w.pts.len() as u64).to_le_bytes());
        for &(x, y) in &w.pts {
            h = fnv1a_from(h, x.to_bits().to_le_bytes());
            h = fnv1a_from(h, y.to_bits().to_le_bytes());
        }
        h = fnv1a_from(h, (w.brks.len() as u64).to_le_bytes());
        for &b in &w.brks {
            h = fnv1a_from(h, (b as u64).to_le_bytes());
        }
        h = fnv1a_from(h, w.km.to_bits().to_le_bytes());
        h = fnv1a_from(h, [w.sea as u8, w.way_type as u8, w.hidden as u8]);
        h = fnv1a_from(h, (w.name.len() as u64).to_le_bytes());
        h = fnv1a_from(h, w.name.bytes());
    }
    h
}

/// A `u64` as 16 hex digits: a GDScript `int` is signed, so an unsigned hash
/// crosses the boundary as text (the `drawn_water_digest` convention).
fn hex(h: u64) -> String {
    format!("{h:016x}")
}

#[godot_api(secondary)]
impl WorldGen {
    /// The ledger's commit counter: the highest ledger `seq` ever issued
    /// (`HistoryLedger::last_seq`). It rises on every recorded commit (a
    /// height snapshot, a way delete or drag, a recorded-only paint or civ
    /// row, a generate or load floor) and on every redo, which records a new
    /// row; an undo never moves it, and nor does removing a row.
    ///
    /// Why the grouped undo needs it: after a group is undone, its engine
    /// steps sit on the redo tail, and a later commit truncates or invalidates
    /// that tail. A commit followed by its own undo leaves every other readout
    /// (depth, labels, bytes) exactly where it was; this counter is the one
    /// value that remembers the commit happened.
    #[func]
    fn undo_commit_seq(&self) -> i64 {
        self.ledger.last_seq() as i64
    }

    /// The identities of the engine's redo offers, **in the order
    /// `redo_last()` would take them**, newest first: element `i` is the step
    /// `redo_labels()[i]` names. An offered undone way drag first (only the top
    /// of the drag stack is offered, exactly as `redo_labels` lists only it),
    /// then the height redo tail while it is current. Empty exactly when
    /// `redo_available()` is false.
    ///
    /// Ids come from `undo::next_step_id`: unique in the process, fresh on
    /// every undo, so an unrelated step committed and undone after a group
    /// shows a new id even when its label is the group's.
    #[func]
    fn redo_offer_ids(&self) -> PackedInt64Array {
        let manual = self.infra.as_ref().map(|t| t.ways.as_slice());
        let drag = context_pick_bridge::drag_redo_ready(&self.way_undo, &self.ledger, manual, self.undo.depth()).map(|r| r.id);
        let tail: Vec<u64> = if self.redo.is_current(self.undo.depth()) { self.redo.steps.ids().collect() } else { Vec::new() };
        drag.into_iter().chain(tail).map(|i| i as i64).collect()
    }

    /// FNV-1a digests of everything an undo group (LLM-5) can change in the
    /// engine, as 16-digit hex strings:
    ///
    /// | Key | What is hashed |
    /// |---|---|
    /// | `field` | the height field, every cell's bit pattern ([`field_digest`]) |
    /// | `column` | the geology column's paintable part (top and sub rock, contact, regolith, painted mask); **omitted** when the world has no column |
    /// | `ways_manual` | the hand-drawn way store, every vertex ([`manual_ways_digest`]) |
    /// | `ways_generated`, `ways_sea` | the generated stores' identities (`fp_generated`, `fp_sea`: not every vertex, see the module doc) |
    /// | `params` | the parameter table's saved form (`params::save_state`, the JSON a project archive stores) |
    /// | `colour_paint` | the colour-paint document JSON, or the hash of `""` when there is no layer (the same `""` `colour_paint_document()` returns) |
    ///
    /// plus `cells` (int). `{}` before any world is generated or loaded.
    ///
    /// Read-only and diagnostic: the shell's undo never decides anything from
    /// it at run time (`undo_group.gd` decides from ledger seqs and entry
    /// identities); probes compare it before and after a group.
    #[func]
    fn edit_state_digest(&self) -> VarDictionary {
        let Some(WorldSource::Generated(ws)) = self.source.as_ref() else {
            return VarDictionary::new();
        };
        let mut d = VarDictionary::new();
        d.set("cells", ws.field.len() as i64);
        d.set("field", hex(field_digest(&ws.field)).as_str());
        if let Some(c) = ws.geology.column() {
            let mut h = fnv1a_from(FNV_OFFSET, c.rock_top.iter().copied());
            h = fnv1a_from(h, c.rock_sub.iter().copied());
            h = fnv1a_from(h, c.contact.iter().flat_map(|v| v.to_bits().to_le_bytes()));
            h = fnv1a_from(h, c.regolith.iter().flat_map(|v| v.to_bits().to_le_bytes()));
            h = match c.painted.as_ref() {
                Some(m) => fnv1a_from(fnv1a_from(h, [1u8]), m.iter().copied()),
                None => fnv1a_from(h, [0u8]),
            };
            d.set("column", hex(h).as_str());
        }
        d.set("ways_manual", hex(manual_ways_digest(self.infra.as_ref().map(|t| t.ways.as_slice()))).as_str());
        d.set("ways_generated", hex(context_pick_bridge::fp_generated(self.civ.as_ref().map(|c| c.ways.as_slice()))).as_str());
        d.set("ways_sea", hex(context_pick_bridge::fp_sea(self.civ.as_ref().map(|c| c.sea_routes.as_slice()))).as_str());
        d.set("params", hex(fnv1a(params::save_state(&self.params).to_string().as_bytes())).as_str());
        let paint = self.colour_paint.as_ref().map(|l| serde_json::to_string(l.layers()).unwrap_or_default()).unwrap_or_default();
        d.set("colour_paint", hex(fnv1a(paint.as_bytes())).as_str());
        d
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cartalith_civ::tools::ManualWayType;

    fn way(pts: &[(f64, f64)]) -> ManualWay {
        ManualWay {
            pts: pts.to_vec(),
            brks: vec![],
            km: 1.5,
            sea: false,
            way_type: ManualWayType::Road,
            name: "Road".into(),
            hidden: false,
        }
    }

    /// Protects: the hash is FNV-1a 64 and not a near miss. The two published
    /// test vectors (the empty string is the offset basis itself; "a" is
    /// 0xaf63dc4c8601ec8c in the FNV reference table) are literals, so a
    /// mutated basis or prime turns this red.
    #[test]
    fn fnv1a_matches_the_published_vectors() {
        assert_eq!(fnv1a(b""), 0xcbf29ce484222325);
        assert_eq!(fnv1a(b"a"), 0xaf63dc4c8601ec8c);
        assert_eq!(fnv1a(b"foobar"), 0x85944171f73967e8);
    }

    /// Protects: one cell changed anywhere moves the field digest, a swap of
    /// two cells moves it (order-sensitive), and an equal field hashes equal
    /// (so "after undo equals before" is a meaningful comparison).
    #[test]
    fn field_digest_sees_one_cell_and_order() {
        let a = vec![0.1f32, 0.2, 0.3, 0.4];
        let mut b = a.clone();
        assert_eq!(field_digest(&a), field_digest(&b));
        b[3] = 0.400_000_04;
        assert_ne!(field_digest(&a), field_digest(&b));
        let swapped = vec![0.2f32, 0.1, 0.3, 0.4];
        assert_ne!(field_digest(&a), field_digest(&swapped));
    }

    /// Protects: bit patterns, not values. `-0.0 == 0.0` in IEEE comparison,
    /// and an undo that wrote back the other zero would not have restored
    /// the field byte for byte.
    #[test]
    fn field_digest_tells_the_two_zeros_apart() {
        assert_ne!(field_digest(&[0.0f32]), field_digest(&[-0.0f32]));
    }

    /// Protects: length is hashed, so a field that is a prefix of another
    /// does not collide with it through the empty tail.
    #[test]
    fn field_digest_hashes_the_length() {
        assert_ne!(field_digest(&[]), field_digest(&[0.0f32]));
        assert_eq!(field_digest(&[]), fnv1a(&0u64.to_le_bytes()));
    }

    /// Protects: the manual-way digest sees an interior vertex (which
    /// `fp_manual`, a cheap identity, deliberately does not), a removed way,
    /// and "no store" versus "an emptied store".
    #[test]
    fn manual_ways_digest_sees_every_vertex() {
        let a = vec![way(&[(0.0, 0.0), (1.0, 1.0), (2.0, 0.0)])];
        let b = vec![way(&[(0.0, 0.0), (1.0, 1.5), (2.0, 0.0)])];
        assert_ne!(manual_ways_digest(Some(&a)), manual_ways_digest(Some(&b)));
        assert_eq!(
            context_pick_bridge::fp_manual(Some(&a)),
            context_pick_bridge::fp_manual(Some(&b)),
            "premise: the cheap identity cannot see the interior vertex"
        );
        assert_ne!(manual_ways_digest(Some(&a)), manual_ways_digest(Some(&[])));
        assert_ne!(manual_ways_digest(None), manual_ways_digest(Some(&[])));
        assert_eq!(manual_ways_digest(Some(&a)), manual_ways_digest(Some(&a.clone())));
    }

    /// Protects: the way's flags and name are in the digest (a hidden way is
    /// a different state from a shown one).
    #[test]
    fn manual_ways_digest_sees_flags_and_name() {
        let a = vec![way(&[(0.0, 0.0), (1.0, 1.0)])];
        let mut b = a.clone();
        b[0].hidden = true;
        assert_ne!(manual_ways_digest(Some(&a)), manual_ways_digest(Some(&b)));
        let mut c = a.clone();
        c[0].name = "Lane".into();
        assert_ne!(manual_ways_digest(Some(&a)), manual_ways_digest(Some(&c)));
    }
}
