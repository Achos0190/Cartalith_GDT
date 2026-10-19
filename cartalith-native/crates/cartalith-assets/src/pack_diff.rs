//! Pack diff and merge (`ASSET_PACK_TOOLS_PROPOSAL.md` AP-9, Ruling CQ):
//! slot by slot, which slots each side fills, whether their images are the
//! same, and which side wins a merge.
//!
//! Both sides are [`AssetDB`]s. The shell's question is "how does this pack
//! differ from my library?", so `cartalith-godot`'s session opens the pack
//! into a scratch library first (AP-1's own `import_pack`, so routing, caps
//! and colour conversion are identical) and then diffs the two registries
//! here. Nothing here needs pixels: every [`LibraryItem`] already carries its
//! content hash (`raster::item_hash`, computed from decoded pixels), so "the
//! same image" is hash equality, and "the same item" adds the transform,
//! which decides the bake.
//!
//! # What is compared, and what deliberately is not
//!
//! - **Compared:** each slot's items as an ordered list of
//!   `(hash, transform)` pairs, and the three pack fields (name, author,
//!   licence, trimmed).
//! - **Not compared: item names and slot metadata.** A pack carries neither:
//!   AP-1 names every item after its slot id and writes `SlotMeta::source`
//!   from the file it came from, so they always differ between a library and
//!   a pack and would bury the real differences. Tags and licences live in
//!   the library (Ruling CQ Q9), not in packs.
//! - **Blank and default pack fields are not equated.** The exporter writes
//!   `"CC0"` for a blank licence (Q8); a library whose licence is blank and a
//!   pack exported from it show a licence difference, which is true.
//!
//! # Merge
//!
//! [`plan_merge`] never deletes: a slot only the library fills is kept under
//! every policy. A slot only the pack fills is added under every policy.
//! Where both fill a slot differently, [`MergePolicy`] decides. **"Keep
//! library" changes nothing the library already fills, item for item, and
//! leaves its pack fields alone**, so merging a pack that adds no new slot
//! leaves the library byte-identical (AP-9's acceptance; tested in the
//! session against the exported bytes).

use crate::library::{AssetDB, LibraryItem};
use crate::slots::Family;
use std::collections::HashSet;

/// How one slot differs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlotChange {
    /// Only side A (the pack) fills it.
    OnlyA,
    /// Only side B (the library) fills it.
    OnlyB,
    /// Both fill it, and their items differ.
    Differs,
}

impl SlotChange {
    /// The lowercase key the shell receives.
    pub fn key(self) -> &'static str {
        match self {
            SlotChange::OnlyA => "only_pack",
            SlotChange::OnlyB => "only_library",
            SlotChange::Differs => "differs",
        }
    }
}

/// One slot's difference.
#[derive(Debug, Clone, PartialEq)]
pub struct SlotDiff {
    /// The slot's uid (the same on both sides: custom uids are slugs of the
    /// set and slot names, so the same pack routes to the same uid).
    pub uid: String,
    /// Its family.
    pub family: Family,
    /// Its display name, from whichever side has it (B first).
    pub name: String,
    /// The change.
    pub change: SlotChange,
    /// Items on side A.
    pub a_items: usize,
    /// Items on side B.
    pub b_items: usize,
    /// How many of A's items show an image B also holds in this slot (hash
    /// equality, transform not considered), for the row's wording.
    pub same_images: usize,
}

/// One pack field that differs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldDiff {
    /// `"name"`, `"author"` or `"licence"`.
    pub field: &'static str,
    /// Side A's value, trimmed.
    pub a: String,
    /// Side B's value, trimmed.
    pub b: String,
}

/// Everything that differs between two libraries.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PackDiff {
    /// Slot differences: B's registry order first, then slots only A has
    /// registered, in A's order.
    pub slots: Vec<SlotDiff>,
    /// Pack-field differences, in the order name, author, licence.
    pub fields: Vec<FieldDiff>,
}

impl PackDiff {
    /// True when nothing differs.
    pub fn is_empty(&self) -> bool {
        self.slots.is_empty() && self.fields.is_empty()
    }
}

/// An item's identity for the diff: its content hash and its transform's
/// three numbers, compared exactly (both sides compute them the same way, so
/// any difference is a real one).
fn same_item(a: &LibraryItem, b: &LibraryItem) -> bool {
    a.hash == b.hash
        && a.transform.scale == b.transform.scale
        && a.transform.pan_x == b.transform.pan_x
        && a.transform.pan_y == b.transform.pan_y
}

/// Diff side `a` (usually a pack opened into a scratch library) against side
/// `b` (usually the live library). See the module doc for what is compared.
pub fn diff_libraries(a: &AssetDB, b: &AssetDB) -> PackDiff {
    let mut out = PackDiff::default();
    let b_order = b.uids_in_order();
    let b_known: HashSet<&str> = b_order.iter().map(String::as_str).collect();
    let mut order: Vec<String> = b_order.clone();
    order.extend(a.uids_in_order().into_iter().filter(|u| !b_known.contains(u.as_str())));
    for uid in order {
        let ai = a.items(&uid);
        let bi = b.items(&uid);
        let change = match (ai.is_empty(), bi.is_empty()) {
            (true, true) => continue,
            (false, true) => SlotChange::OnlyA,
            (true, false) => SlotChange::OnlyB,
            (false, false) => {
                if ai.len() == bi.len() && ai.iter().zip(bi).all(|(x, y)| same_item(x, y)) {
                    continue;
                }
                SlotChange::Differs
            }
        };
        let slot = b.get(&uid).or_else(|| a.get(&uid)).expect("uid came from one of the two registries");
        let b_hashes: HashSet<&str> = bi.iter().map(|i| i.hash.as_str()).collect();
        out.slots.push(SlotDiff {
            uid: uid.clone(),
            family: slot.family,
            name: slot.name.clone(),
            change,
            a_items: ai.len(),
            b_items: bi.len(),
            same_images: ai.iter().filter(|i| b_hashes.contains(i.hash.as_str())).count(),
        });
    }
    let fields: [(&'static str, &str, &str); 3] = [
        ("name", &a.pack.name, &b.pack.name),
        ("author", &a.pack.author, &b.pack.author),
        ("licence", &a.pack.license, &b.pack.license),
    ];
    for (field, av, bv) in fields {
        if av.trim() != bv.trim() {
            out.fields.push(FieldDiff { field, a: av.trim().to_string(), b: bv.trim().to_string() });
        }
    }
    out
}

/// Which side wins where both fill a slot differently.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MergePolicy {
    /// The library's items stay exactly as they are, and so do its pack
    /// fields.
    KeepB,
    /// The pack's items replace the library's in that slot, and the pack's
    /// non-blank fields replace the library's.
    TakeA,
    /// The pack's images the library does not already hold (by hash) are
    /// appended as new variants; a single-image slot keeps the library's.
    /// Blank library fields are filled from the pack.
    KeepBoth,
}

impl MergePolicy {
    /// The policy for the shell's key: `"keep_library"`, `"take_pack"`,
    /// `"keep_both"`; `None` for anything else.
    pub fn from_key(key: &str) -> Option<MergePolicy> {
        match key {
            "keep_library" => Some(MergePolicy::KeepB),
            "take_pack" => Some(MergePolicy::TakeA),
            "keep_both" => Some(MergePolicy::KeepBoth),
            _ => None,
        }
    }
}

/// What a merge does to one slot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MergeAction {
    /// Copy every A item in (the slot is empty in B).
    Add,
    /// Empty B's slot, then copy every A item in.
    Replace,
    /// Append these A items (indices into A's slot), which B does not hold.
    Append(Vec<usize>),
    /// Leave B's slot as it is.
    Keep,
}

/// One slot's merge step.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MergeStep {
    /// The slot.
    pub uid: String,
    /// What happens to it.
    pub action: MergeAction,
    /// Why, when the policy could not do what its name says (a single-image
    /// slot under [`MergePolicy::KeepBoth`]).
    pub note: Option<String>,
}

/// The whole merge: per-slot steps (one per [`PackDiff::slots`] row, same
/// order) and the pack fields to write (field, new value).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MergePlan {
    /// One per diff row.
    pub steps: Vec<MergeStep>,
    /// Pack fields to overwrite on B.
    pub fields: Vec<(&'static str, String)>,
}

/// Decide a merge of `a` into `b` from their `diff`. Pure: the session
/// carries it out with pixels.
pub fn plan_merge(a: &AssetDB, b: &AssetDB, diff: &PackDiff, policy: MergePolicy) -> MergePlan {
    let mut plan = MergePlan::default();
    for d in &diff.slots {
        let (action, note) = match (d.change, policy) {
            (SlotChange::OnlyA, _) => (MergeAction::Add, None),
            (SlotChange::OnlyB, _) | (SlotChange::Differs, MergePolicy::KeepB) => (MergeAction::Keep, None),
            (SlotChange::Differs, MergePolicy::TakeA) => (MergeAction::Replace, None),
            (SlotChange::Differs, MergePolicy::KeepBoth) => {
                if !d.family.is_multi() {
                    (
                        MergeAction::Keep,
                        Some(format!("{} holds one image, so the library's was kept", d.name)),
                    )
                } else {
                    let held: HashSet<&str> = b.items(&d.uid).iter().map(|i| i.hash.as_str()).collect();
                    let new: Vec<usize> = a
                        .items(&d.uid)
                        .iter()
                        .enumerate()
                        .filter(|(_, i)| !held.contains(i.hash.as_str()))
                        .map(|(k, _)| k)
                        .collect();
                    if new.is_empty() { (MergeAction::Keep, None) } else { (MergeAction::Append(new), None) }
                }
            }
        };
        plan.steps.push(MergeStep { uid: d.uid.clone(), action, note });
    }
    for f in &diff.fields {
        let write = match policy {
            MergePolicy::KeepB => false,
            MergePolicy::TakeA => !f.a.is_empty(),
            MergePolicy::KeepBoth => f.b.is_empty() && !f.a.is_empty(),
        };
        if write {
            plan.fields.push((f.field, f.a.clone()));
        }
    }
    plan
}

/// The diff and the merge plan on registries built by hand (hashes stand
/// in for pixels, which this module never reads).
#[cfg(test)]
mod tests {
    use super::*;
    use crate::library::ItemTransform;

    fn item(hash: &str) -> LibraryItem {
        LibraryItem::new("x", hash)
    }

    /// A library with `mountain` = [h1, h2], `grass` = [g1], one custom slot.
    fn lib() -> AssetDB {
        let mut db = AssetDB::new();
        db.add_item("icons:mountain", item("h1"));
        db.add_item("icons:mountain", item("h2"));
        db.add_item("textures:grass", item("g1"));
        let uid = db.add_custom_slot("Lighthouse", Some("Naval")).uid.clone();
        db.add_item(&uid, item("c1"));
        db.pack.name = "P".into();
        db
    }

    /// Protects: AP-9's first bar, a library diffed against itself is empty.
    #[test]
    fn a_library_against_itself_is_empty() {
        let a = lib();
        assert!(diff_libraries(&a, &a.clone()).is_empty());
    }

    /// Protects: each change kind, the transform counting as a difference,
    /// the field diff, and the row order (B's registry, then A-only).
    #[test]
    fn every_change_kind_is_reported_in_order() {
        let mut a = lib();
        let mut b = lib();
        // A changes mountain's second image, B fills hill alone, A has an
        // extra custom slot, A moves grass's pan.
        a.item_mut("icons:mountain", 1).unwrap().hash = "h9".into();
        b.add_item("icons:hill", item("k1"));
        let extra = a.add_custom_slot("Buoy", Some("Naval")).uid.clone();
        a.add_item(&extra, item("c2"));
        a.item_mut("textures:grass", 0).unwrap().transform = ItemTransform { scale: 1.0, pan_x: 2.0, pan_y: 0.0 };
        a.pack.author = "Someone".into();
        let d = diff_libraries(&a, &b);
        let rows: Vec<(&str, SlotChange, usize, usize, usize)> =
            d.slots.iter().map(|s| (s.uid.as_str(), s.change, s.a_items, s.b_items, s.same_images)).collect();
        assert_eq!(
            rows,
            vec![
                ("textures:grass", SlotChange::Differs, 1, 1, 1),
                ("icons:mountain", SlotChange::Differs, 2, 2, 1),
                ("icons:hill", SlotChange::OnlyB, 0, 1, 0),
                ("custom:naval/buoy", SlotChange::OnlyA, 1, 0, 0),
            ]
        );
        assert_eq!(d.fields, vec![FieldDiff { field: "author", a: "Someone".into(), b: "".into() }]);
    }

    /// Protects: the three policies' decisions, including Keep library
    /// touching nothing B fills, Keep both appending only new hashes, and a
    /// single-image slot under Keep both keeping B's with a note.
    #[test]
    fn the_policies_decide_as_documented() {
        let mut a = lib();
        let b = lib();
        a.item_mut("icons:mountain", 1).unwrap().hash = "h9".into();
        a.item_mut("textures:grass", 0).unwrap().hash = "g9".into();
        let extra = a.add_custom_slot("Buoy", Some("Naval")).uid.clone();
        a.add_item(&extra, item("c2"));
        a.pack.name = "Q".into();
        let d = diff_libraries(&a, &b);
        let acts = |p: MergePolicy| -> Vec<(String, MergeAction)> {
            plan_merge(&a, &b, &d, p).steps.into_iter().map(|s| (s.uid, s.action)).collect()
        };
        assert_eq!(
            acts(MergePolicy::KeepB),
            vec![
                ("textures:grass".into(), MergeAction::Keep),
                ("icons:mountain".into(), MergeAction::Keep),
                ("custom:naval/buoy".into(), MergeAction::Add),
            ]
        );
        assert_eq!(
            acts(MergePolicy::TakeA),
            vec![
                ("textures:grass".into(), MergeAction::Replace),
                ("icons:mountain".into(), MergeAction::Replace),
                ("custom:naval/buoy".into(), MergeAction::Add),
            ]
        );
        let both = plan_merge(&a, &b, &d, MergePolicy::KeepBoth);
        assert_eq!(both.steps[0].action, MergeAction::Keep);
        assert!(both.steps[0].note.as_deref().unwrap().contains("one image"));
        assert_eq!(both.steps[1].action, MergeAction::Append(vec![1]));
        assert!(plan_merge(&a, &b, &d, MergePolicy::KeepB).fields.is_empty());
        assert_eq!(plan_merge(&a, &b, &d, MergePolicy::TakeA).fields, vec![("name", "Q".to_string())]);
        assert!(both.fields.is_empty(), "keep both fills only blank fields; B's name is not blank");
    }

    /// Protects: the policy keys the shell sends.
    #[test]
    fn policy_keys() {
        assert_eq!(MergePolicy::from_key("keep_library"), Some(MergePolicy::KeepB));
        assert_eq!(MergePolicy::from_key("take_pack"), Some(MergePolicy::TakeA));
        assert_eq!(MergePolicy::from_key("keep_both"), Some(MergePolicy::KeepBoth));
        assert_eq!(MergePolicy::from_key("keep"), None);
        assert_eq!(SlotChange::OnlyA.key(), "only_pack");
    }
}
