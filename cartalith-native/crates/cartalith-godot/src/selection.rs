//! One selection **set** per entity kind — step one of the owner's ruling in
//! `LARGE_ITEM_RULINGS.md`: *"Selection sets → clipboard → commands, in that
//! order. Step one — a selection set per entity kind, replacing the three
//! unrelated single-`i64` selections — is independently valuable and pays for
//! itself even if the clipboard never lands."*
//!
//! The three scalars this replaces were `IconEditor::selected`,
//! `LabelEditSession::selected()` (through `LabelBridge`) and
//! `SculptEditor::selected`, surfaced as `icon_get_selected`,
//! `label_get_selected` and `sculpt_get_selected_stamp`. Each is now
//! [`SelectionSet::primary`] over one of these, so **every existing caller of
//! those three getters keeps working unchanged**: a set holding zero or one
//! member answers exactly what the scalar answered.
//!
//! Deliberately **free of any `godot` dependency**, the same isolation
//! `sculpt_bridge.rs` and `icon_bridge.rs` already keep: `lib.rs` owns the
//! `#[func]` surface and the `PackedInt64Array` conversion, this module owns
//! the set semantics, and the tests below run under `cargo test` with no Godot
//! runtime involved.
//!
//! ## The ordering is load-bearing, and it is recency, not index order
//!
//! Members are stored in the order they were added, so the **last** one is the
//! primary — the one a resize handle, an edit snapshot or a parameter block
//! belongs to, and the one the scalar getters report. That is what makes a
//! plain click, a Ctrl-click and a Shift-click all agree on "the one I just
//! touched". [`SelectionSet::sorted`] is the ascending view for anything that
//! wants the set as a set (the `*_get_selection()` bindings, and whatever
//! clipboard step two brings); nothing outside this module sees the internal
//! order.
//!
//! ## What it deliberately does not do
//!
//! No entity data, no clipboard, no commands. It holds `usize` indices into
//! somebody else's `Vec` and knows one thing about that `Vec`: what happens to
//! the indices when an element is removed from it
//! ([`SelectionSet::retain_after_remove`]). Validation against the list's
//! length stays with the owner of the list, which is the only place that knows
//! it.

/// What a click does to a selection set — the three modes the shell's own
/// established convention already uses (`asset_library_window.gd`'s grid:
/// plain click replaces, Ctrl/Cmd-click toggles, Shift-click extends a range),
/// which is also the platform-conventional set.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectMode {
    /// Plain click: the set becomes exactly `{index}`.
    Replace,
    /// Ctrl/Cmd-click: add `index` if absent (and make it primary), remove it
    /// if present.
    Toggle,
    /// Shift-click: the inclusive range between the current primary (the
    /// anchor) and `index`, replacing what was there — `asset_library_window.
    /// gd:2174-2179`'s own `_selected.clear()`-then-fill. With nothing
    /// selected there is no anchor, so it degrades to [`SelectMode::Replace`].
    Extend,
}

impl SelectMode {
    /// The wire encoding the `#[func]` bindings use: `0` replace, `1` toggle,
    /// `2` extend. Anything else is [`SelectMode::Replace`] — a caller that
    /// sends garbage gets the plain-click behaviour rather than a silently
    /// ignored click.
    pub fn from_i64(v: i64) -> Self {
        match v {
            1 => SelectMode::Toggle,
            2 => SelectMode::Extend,
            _ => SelectMode::Replace,
        }
    }
}

/// An ordered set of indices into one entity list. See the module doc.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SelectionSet {
    /// Insertion order, last = primary. Never holds a duplicate.
    items: Vec<usize>,
}

impl SelectionSet {
    pub fn new() -> Self {
        Self::default()
    }

    /// The member a single-selection operation acts on — the most recently
    /// added one. This is what `icon_get_selected`/`label_get_selected`/
    /// `sculpt_get_selected_stamp` report, and it is `None` for an empty set,
    /// which is the `-1` all three of them already returned.
    pub fn primary(&self) -> Option<usize> {
        self.items.last().copied()
    }

    pub fn contains(&self, index: usize) -> bool {
        self.items.contains(&index)
    }

    /// Every member, ascending. The public view — see the module doc on why
    /// the internal order is recency and this one is not.
    pub fn sorted(&self) -> Vec<usize> {
        let mut v = self.items.clone();
        v.sort_unstable();
        v
    }

    pub fn clear(&mut self) {
        self.items.clear();
    }

    /// The set becomes exactly `{index}`.
    pub fn replace(&mut self, index: usize) {
        self.items.clear();
        self.items.push(index);
    }

    /// Adds `index` (making it primary) if absent, removes it if present.
    /// Returns whether `index` is selected *after* the call.
    pub fn toggle(&mut self, index: usize) -> bool {
        match self.items.iter().position(|&i| i == index) {
            Some(at) => {
                self.items.remove(at);
                false
            }
            None => {
                self.items.push(index);
                true
            }
        }
    }

    /// The inclusive range between the current primary and `index`, in
    /// ascending order with `index` moved last so it stays the primary (the
    /// anchor for a *further* Shift-click, which is what makes a run of them
    /// pivot around the original anchor rather than crawl). With nothing
    /// selected this is [`SelectionSet::replace`].
    pub fn extend_to(&mut self, index: usize) {
        let Some(anchor) = self.primary() else {
            self.replace(index);
            return;
        };
        let (lo, hi) = if anchor <= index { (anchor, index) } else { (index, anchor) };
        self.items.clear();
        self.items.extend((lo..=hi).filter(|&i| i != index));
        self.items.push(index);
    }

    /// Applies one click.
    pub fn apply(&mut self, mode: SelectMode, index: usize) {
        match mode {
            SelectMode::Replace => self.replace(index),
            SelectMode::Toggle => {
                self.toggle(index);
            }
            SelectMode::Extend => self.extend_to(index),
        }
    }

    /// Replaces the whole set from an arbitrary caller-supplied list, keeping
    /// it ascending and duplicate-free. `len` is the owning list's length:
    /// anything at or past it is dropped rather than rejecting the whole call,
    /// so a stale index from a shell that has not refreshed cannot leave a
    /// member pointing at nothing. Returns whether every requested index was
    /// in range.
    ///
    /// The `#[func]` callers map a negative `i64` to `usize::MAX` rather than
    /// dropping it before it reaches here, so a `[-1]` from GDScript is
    /// reported out of range instead of silently counting as valid.
    pub fn set_from(&mut self, indices: impl IntoIterator<Item = usize>, len: usize) -> bool {
        let mut all_valid = true;
        let mut v: Vec<usize> = indices
            .into_iter()
            .filter(|&i| {
                let ok = i < len;
                all_valid &= ok;
                ok
            })
            .collect();
        v.sort_unstable();
        v.dedup();
        self.items = v;
        all_valid
    }

    /// Every index, ascending — `set_from(0..len, len)`, the shape a future
    /// `Select all` wants and the reason this type exists at all.
    pub fn select_all(&mut self, len: usize) {
        self.items = (0..len).collect();
    }

    /// Re-points the set after `Vec::remove(removed)` on the owning list:
    /// drops `removed` itself and decrements every member past it, so each
    /// surviving member still names the same logical entity in the now-shorter
    /// list. `IconEditor::delete`'s own rule, generalised —
    /// `sculpt_delete_stamp` previously only handled the equal case and left a
    /// later selection silently naming its neighbour (see this call's own site
    /// in `lib.rs` for the disclosure).
    ///
    /// At zero or one member this is exactly what the scalars did: the removed
    /// one clears, a later one shifts down.
    pub fn retain_after_remove(&mut self, removed: usize) {
        self.items.retain(|&i| i != removed);
        for i in &mut self.items {
            if *i > removed {
                *i -= 1;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_empty_set_answers_the_scalar_getters_the_way_none_did() {
        let s = SelectionSet::new();
        assert_eq!(s.primary(), None);
        assert!(s.sorted().is_empty());
    }

    #[test]
    fn replace_is_a_single_selection() {
        let mut s = SelectionSet::new();
        s.replace(3);
        s.replace(7);
        assert_eq!(s.sorted(), vec![7]);
        assert_eq!(s.primary(), Some(7));
    }

    #[test]
    fn toggle_adds_then_removes_and_reports_which() {
        let mut s = SelectionSet::new();
        assert!(s.toggle(2));
        assert!(s.toggle(5));
        assert_eq!(s.primary(), Some(5), "the most recent add is primary");
        assert_eq!(s.sorted(), vec![2, 5]);
        assert!(!s.toggle(5));
        assert_eq!(s.sorted(), vec![2]);
        assert_eq!(s.primary(), Some(2), "removing the primary promotes the one before it");
    }

    #[test]
    fn toggling_away_the_last_member_empties_the_set() {
        let mut s = SelectionSet::new();
        s.replace(4);
        assert!(!s.toggle(4));
        assert!(s.sorted().is_empty());
        assert_eq!(s.primary(), None);
    }

    #[test]
    fn extend_covers_the_inclusive_range_in_both_directions() {
        let mut s = SelectionSet::new();
        s.replace(2);
        s.extend_to(5);
        assert_eq!(s.sorted(), vec![2, 3, 4, 5]);
        assert_eq!(s.primary(), Some(5));

        let mut back = SelectionSet::new();
        back.replace(5);
        back.extend_to(2);
        assert_eq!(back.sorted(), vec![2, 3, 4, 5]);
        assert_eq!(back.primary(), Some(2), "the clicked end is the new anchor");
    }

    #[test]
    fn extend_with_nothing_selected_is_a_plain_click() {
        let mut s = SelectionSet::new();
        s.extend_to(6);
        assert_eq!(s.sorted(), vec![6]);
        assert_eq!(s.primary(), Some(6));
    }

    #[test]
    fn extend_replaces_rather_than_unions_matching_the_shells_own_grid() {
        // `asset_library_window.gd:2177` clears before filling the range.
        let mut s = SelectionSet::new();
        s.replace(0);
        s.toggle(9);
        s.extend_to(11);
        assert_eq!(s.sorted(), vec![9, 10, 11], "0 is gone, not kept");
    }

    #[test]
    fn apply_routes_the_three_modes() {
        let mut s = SelectionSet::new();
        s.apply(SelectMode::Replace, 1);
        s.apply(SelectMode::Toggle, 4);
        assert_eq!(s.sorted(), vec![1, 4]);
        s.apply(SelectMode::Extend, 6);
        assert_eq!(s.sorted(), vec![4, 5, 6]);
    }

    #[test]
    fn the_wire_encoding_maps_and_falls_back_to_replace() {
        assert_eq!(SelectMode::from_i64(0), SelectMode::Replace);
        assert_eq!(SelectMode::from_i64(1), SelectMode::Toggle);
        assert_eq!(SelectMode::from_i64(2), SelectMode::Extend);
        assert_eq!(SelectMode::from_i64(99), SelectMode::Replace);
        assert_eq!(SelectMode::from_i64(-1), SelectMode::Replace);
    }

    #[test]
    fn set_from_drops_out_of_range_and_reports_it() {
        let mut s = SelectionSet::new();
        assert!(s.set_from([2, 0, 1], 3));
        assert_eq!(s.sorted(), vec![0, 1, 2]);
        assert!(!s.set_from([1, 9], 3), "9 is past the end");
        assert_eq!(s.sorted(), vec![1]);
    }

    #[test]
    fn set_from_dedupes() {
        let mut s = SelectionSet::new();
        assert!(s.set_from([2, 2, 1, 1, 1], 4));
        assert_eq!(s.sorted(), vec![1, 2]);
    }

    #[test]
    fn select_all_takes_the_whole_list_and_an_empty_one_selects_nothing() {
        let mut s = SelectionSet::new();
        s.select_all(3);
        assert_eq!(s.sorted(), vec![0, 1, 2]);
        s.select_all(0);
        assert!(s.sorted().is_empty());
        assert_eq!(s.primary(), None);
    }

    #[test]
    fn retain_after_remove_reproduces_the_scalar_rule_at_one_member() {
        // The removed one clears -- `IconEditor::delete`'s `Some(s) if s == index`.
        let mut s = SelectionSet::new();
        s.replace(2);
        s.retain_after_remove(2);
        assert_eq!(s.primary(), None);

        // A later one shifts down -- its `Some(s) if s > index => Some(s - 1)`.
        let mut later = SelectionSet::new();
        later.replace(5);
        later.retain_after_remove(2);
        assert_eq!(later.primary(), Some(4));

        // An earlier one is untouched -- its `other => other`.
        let mut earlier = SelectionSet::new();
        earlier.replace(1);
        earlier.retain_after_remove(4);
        assert_eq!(earlier.primary(), Some(1));
    }

    #[test]
    fn retain_after_remove_shifts_every_member_of_a_real_set() {
        let mut s = SelectionSet::new();
        s.set_from([1, 3, 5], 6);
        s.retain_after_remove(3);
        assert_eq!(s.sorted(), vec![1, 4], "3 is gone, 5 became 4, 1 stayed");
    }

    #[test]
    fn retain_after_remove_keeps_recency_order_for_the_survivors() {
        let mut s = SelectionSet::new();
        s.replace(7);
        s.toggle(2);
        assert_eq!(s.primary(), Some(2));
        s.retain_after_remove(4);
        assert_eq!(s.primary(), Some(2), "2 was primary and is before the removal");
        assert_eq!(s.sorted(), vec![2, 6]);
    }
}
