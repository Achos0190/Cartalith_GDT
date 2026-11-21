//! F3 / F4 (owner findings 2026-11-21) — the Godot surface for editing **generated** labels.
//!
//! Until now only hand-placed labels could be renamed, moved or removed, because every edit call addresses
//! `LabelBridge::labels` by index and a generated label lives in `LabelBridge::generated`, which a regenerate rebuilds.
//! These calls address a generated label by its [`crate::label_bridge::auto_key`] and write an *override over it*
//! (`LabelBridge::overrides`): the override replaces the generated label where it is drawn and picked, so there is never a
//! second label, and it survives a regenerate because the key is built from what the generator itself produced.
//!
//! Every write is one undo step through `anno_scope` (the override map is part of the `AnnoSnap::Labels` snapshot) and is
//! saved with the labels (`annotations/labels.json`'s `auto_overrides`).

use godot::prelude::*;

use crate::{label_bridge, variant_to_num, variant_to_string, WorldGen};

/// One generated label's state as the panel and the drag need it: `label_dict`'s fields plus `key`, `class`, `hidden`,
/// `edited` (an override exists).
fn row(key: &str, lb: &cartalith_civ::labels::MapLabel, hidden: bool, edited: bool) -> VarDictionary {
    let mut d = crate::label_dict(lb);
    d.set("key", key);
    d.set("class", lb.class.key());
    d.set("hidden", hidden);
    d.set("edited", edited);
    d
}

#[godot_api(secondary)]
impl WorldGen {
    /// Every generated label, hidden ones included, each with its `key`, `hidden` and `edited` flags. Empty before the
    /// labelling pass has run.
    #[func]
    fn auto_label_list(&self) -> Array<VarDictionary> {
        let Some(b) = self.labels.as_ref() else { return Array::new() };
        b.auto_all().map(|(k, lb, hidden)| row(&k, lb, hidden, b.overrides.contains_key(&k))).collect()
    }

    /// One generated label by `key` (`{}` when the current run does not hold it).
    #[func]
    fn auto_label_get(&self, key: GString) -> VarDictionary {
        let key = key.to_string();
        let Some(b) = self.labels.as_ref() else { return VarDictionary::new() };
        b.auto_all().find(|(k, ..)| *k == key).map_or_else(VarDictionary::new, |(k, lb, hidden)| {
            row(&k, lb, hidden, b.overrides.contains_key(&k))
        })
    }

    /// The topmost visible generated label under `(gx, gy)`, as its `key`, or `""` on a miss. Selects nothing; same box
    /// model and arguments as `label_hit_test`.
    #[func]
    fn auto_label_hit(&self, gx: f64, gy: f64, px_per_cell: f64, zoom: f64) -> GString {
        let hit = self.labels.as_ref().and_then(|b| b.auto_hit(gx, gy, px_per_cell, zoom));
        GString::from(hit.unwrap_or_default().as_str())
    }

    /// Edits generated label `key`: `text`, `x`, `y`, `size`, `size_mode`, `arc`, `angle`, `font`, `color` (as
    /// `label_set`) and `hidden` (bool). Returns `label_set`'s `{ok, rejected, clamped}`; `ok` is false for a key the
    /// current run does not hold.
    #[func]
    fn auto_label_set(&mut self, key: GString, values: VarDictionary) -> VarDictionary {
        let name = self.auto_label_name(&key.to_string());
        let mut keys: Vec<String> = values.keys_array().iter_shared().map(|k| k.to_string()).collect();
        keys.sort();
        let verb = if values.contains_key("hidden") {
            "Hide or show"
        } else if values.contains_key("text") {
            "Rename"
        } else if keys.iter().all(|k| k == "x" || k == "y") {
            "Move"
        } else {
            "Edit"
        };
        let label = crate::annotation_bridge::row_label(verb, "label", &name);
        let stream = Some(format!("auto:{key}:{}", keys.join(",")));
        self.anno_scope(crate::annotation_undo::AnnoStore::Labels, label, stream, None, |s| s.auto_label_set_inner(&key.to_string(), values))
    }

    /// [`Self::auto_label_set`]'s mutation.
    fn auto_label_set_inner(&mut self, key: &str, values: VarDictionary) -> VarDictionary {
        let mut rejected = PackedStringArray::new();
        let mut clamped = PackedStringArray::new();
        let not_ok = |r: &PackedStringArray, c: &PackedStringArray| dict! { "ok" => false, "rejected" => r, "clamped" => c };
        let Some(o) = self.labels.as_mut().and_then(|b| b.auto_edit(key)) else { return not_ok(&rejected, &clamped) };
        for (k, v) in values.iter_shared() {
            let k = k.to_string();
            let lb = &mut o.label;
            let outcome = match k.as_str() {
                "text" => variant_to_string(&v).map_or(label_bridge::Outcome::Rejected, |s| label_bridge::set_text(lb, s)),
                "font" => variant_to_string(&v).map_or(label_bridge::Outcome::Rejected, |s| label_bridge::set_font(lb, s)),
                "color" => variant_to_string(&v).map_or(label_bridge::Outcome::Rejected, |s| label_bridge::set_color(lb, s)),
                "size_mode" => variant_to_string(&v).map_or(label_bridge::Outcome::Rejected, |s| label_bridge::set_size_mode(lb, &s)),
                "size" => variant_to_num(&v).map_or(label_bridge::Outcome::Rejected, |n| label_bridge::set_size(lb, n)),
                "arc" => variant_to_num(&v).map_or(label_bridge::Outcome::Rejected, |n| label_bridge::set_arc(lb, n)),
                "angle" => variant_to_num(&v).map_or(label_bridge::Outcome::Rejected, |n| label_bridge::set_angle(lb, n)),
                "x" | "y" => match variant_to_num(&v).filter(|n| n.is_finite()) {
                    Some(n) => {
                        if k == "x" { lb.x = n } else { lb.y = n }
                        label_bridge::Outcome::Applied
                    }
                    None => label_bridge::Outcome::Rejected,
                },
                "hidden" => match v.try_to::<bool>() {
                    Ok(h) => {
                        o.hidden = h;
                        label_bridge::Outcome::Applied
                    }
                    Err(_) => label_bridge::Outcome::Rejected,
                },
                _ => label_bridge::Outcome::Rejected,
            };
            match outcome {
                label_bridge::Outcome::Applied => {}
                label_bridge::Outcome::Clamped => clamped.push(&GString::from(&k)),
                label_bridge::Outcome::Rejected => rejected.push(&GString::from(&k)),
            }
        }
        dict! { "ok" => true, "rejected" => &rejected, "clamped" => &clamped }
    }

    /// Returns generated label `key` to how the generator made it (drops the override). `false` if it had none.
    #[func]
    fn auto_label_reset(&mut self, key: GString) -> bool {
        let label = crate::annotation_bridge::row_label("Restore", "label", &self.auto_label_name(&key.to_string()));
        self.anno_scope(crate::annotation_undo::AnnoStore::Labels, label, None, None, |s| {
            s.labels.as_mut().is_some_and(|b| b.auto_reset(&key.to_string()))
        })
    }

    /// Drops every override (one undo step); returns how many there were.
    #[func]
    fn auto_label_reset_all(&mut self) -> i64 {
        self.anno_scope(crate::annotation_undo::AnnoStore::Labels, "Restore all generated labels".to_string(), None, None, |s| {
            s.labels.as_mut().map_or(0, |b| std::mem::take(&mut b.overrides).len() as i64)
        })
    }

    /// The text generated label `key` currently shows, for an undo row's name; `""` when unknown.
    fn auto_label_name(&self, key: &str) -> String {
        self.labels.as_ref().and_then(|b| b.auto_all().find(|(k, ..)| k == key).map(|(_, lb, _)| lb.name.clone())).unwrap_or_default()
    }
}
