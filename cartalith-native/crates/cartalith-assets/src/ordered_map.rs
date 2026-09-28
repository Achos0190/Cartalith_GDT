//! A string-keyed map that preserves JSON document order.
//!
//! **Why this exists rather than `BTreeMap` or `serde_json::Map`.** The
//! reference's `parsePackManifest` emits its "unknown slot" warnings by
//! iterating the manifest's own objects (`for(const k in rawTex)`), and
//! JavaScript iterates string keys in *insertion* order. A pack's warning list
//! is therefore a function of the order the author wrote the keys in, and this
//! port golden-matches those lists exactly (`tests/golden_parity_pack_manifest.rs`).
//! `BTreeMap` would sort them; `serde_json::Map` only preserves order behind
//! serde_json's `preserve_order` feature, which — because Cargo unifies
//! features across a workspace — would silently change `cartalith-io`'s
//! `serde_json` behaviour too. Forty lines here keep the blast radius at zero.
//!
//! Duplicate keys follow JavaScript object-assignment semantics: the *first*
//! position is kept, the *last* value wins.

use serde::de::{MapAccess, Visitor};
use serde::ser::SerializeMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::marker::PhantomData;

/// An insertion-ordered `String -> V` map.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrderedMap<V>(Vec<(String, V)>);

impl<V> Default for OrderedMap<V> {
    /// An empty map, same as [`OrderedMap::new`].
    fn default() -> Self {
        OrderedMap(Vec::new())
    }
}

impl<V> OrderedMap<V> {
    /// An empty map.
    pub fn new() -> Self {
        Self::default()
    }

    /// Number of entries.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Whether the map holds no entries.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// The value stored under `key`, if any.
    pub fn get(&self, key: &str) -> Option<&V> {
        self.0.iter().find(|(k, _)| k == key).map(|(_, v)| v)
    }

    /// Whether `key` is present.
    pub fn contains_key(&self, key: &str) -> bool {
        self.0.iter().any(|(k, _)| k == key)
    }

    /// Insert, keeping an existing key's original position and replacing its
    /// value — JavaScript's own object-assignment behaviour.
    pub fn insert(&mut self, key: impl Into<String>, value: V) {
        let key = key.into();
        match self.0.iter_mut().find(|(k, _)| *k == key) {
            Some(slot) => slot.1 = value,
            None => self.0.push((key, value)),
        }
    }

    /// A mutable reference to the value stored under `key`, if any.
    pub fn get_mut(&mut self, key: &str) -> Option<&mut V> {
        self.0.iter_mut().find(|(k, _)| k == key).map(|(_, v)| v)
    }

    /// Remove `key`, returning its value if it was present — JavaScript's
    /// `delete`. A later [`insert`](Self::insert) of the same key is a fresh
    /// insertion at the end, matching `delete`-then-reassign's effect on
    /// `Object.keys` order.
    pub fn remove(&mut self, key: &str) -> Option<V> {
        let pos = self.0.iter().position(|(k, _)| k == key)?;
        Some(self.0.remove(pos).1)
    }

    /// Entries in insertion order.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &V)> {
        self.0.iter().map(|(k, v)| (k.as_str(), v))
    }

    /// Mutable entries in insertion order.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&str, &mut V)> {
        self.0.iter_mut().map(|(k, v)| (k.as_str(), v))
    }

    /// Keys in insertion order.
    pub fn keys(&self) -> impl Iterator<Item = &str> {
        self.0.iter().map(|(k, _)| k.as_str())
    }

    /// Values in insertion order.
    pub fn values(&self) -> impl Iterator<Item = &V> {
        self.0.iter().map(|(_, v)| v)
    }
}

impl<V> FromIterator<(String, V)> for OrderedMap<V> {
    /// Builds a map by inserting each pair in iteration order, so a later
    /// duplicate key overwrites the earlier value in place (`insert`'s
    /// JavaScript-assignment semantics), not appended.
    fn from_iter<I: IntoIterator<Item = (String, V)>>(iter: I) -> Self {
        let mut out = OrderedMap::new();
        for (k, v) in iter {
            out.insert(k, v);
        }
        out
    }
}

impl<'a, V> IntoIterator for &'a OrderedMap<V> {
    type Item = (&'a str, &'a V);
    type IntoIter = Box<dyn Iterator<Item = (&'a str, &'a V)> + 'a>;
    /// `for (k, v) in &map` -- delegates to [`OrderedMap::iter`], boxed
    /// because the borrowed-str/associated-type shape has no named type.
    fn into_iter(self) -> Self::IntoIter {
        Box::new(self.iter())
    }
}

impl<V: Serialize> Serialize for OrderedMap<V> {
    /// Serializes as a JSON object with keys in insertion order -- the
    /// property this whole type exists to guarantee.
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(self.0.len()))?;
        for (k, v) in &self.0 {
            map.serialize_entry(k, v)?;
        }
        map.end()
    }
}

/// [`serde`]'s visitor for [`OrderedMap`]'s [`Deserialize`] impl below.
struct OrderedMapVisitor<V>(PhantomData<V>);

impl<'de, V: Deserialize<'de>> Visitor<'de> for OrderedMapVisitor<V> {
    type Value = OrderedMap<V>;

    /// The error-message description `serde` prints on a type mismatch.
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("a JSON object")
    }

    /// Reads entries in the document's own order and inserts them with
    /// [`OrderedMap::insert`], so a duplicate key keeps its first position
    /// and takes its last value -- JavaScript's own object-literal rule.
    fn visit_map<M: MapAccess<'de>>(self, mut access: M) -> Result<Self::Value, M::Error> {
        let mut out = OrderedMap::new();
        while let Some((k, v)) = access.next_entry::<String, V>()? {
            out.insert(k, v);
        }
        Ok(out)
    }
}

impl<'de, V: Deserialize<'de>> Deserialize<'de> for OrderedMap<V> {
    /// Deserializes a JSON object, preserving its source key order.
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_map(OrderedMapVisitor(PhantomData))
    }
}

/// [`OrderedMap`]'s order-preservation and JS-assignment-semantics contract.
#[cfg(test)]
mod tests {
    use super::*;

    /// Protects: a JSON object round-trips through [`OrderedMap`] with its
    /// source key order intact, both on read (`keys()`) and on write
    /// (re-serializing must reproduce the original text).
    #[test]
    fn preserves_document_order_through_a_round_trip() {
        let src = r#"{"zebra":"z","apple":"a","mango":"m"}"#;
        let m: OrderedMap<String> = serde_json::from_str(src).unwrap();
        assert_eq!(m.keys().collect::<Vec<_>>(), ["zebra", "apple", "mango"]);
        assert_eq!(serde_json::to_string(&m).unwrap(), src);
    }

    /// Protects: `{a:1, b:2, a:3}`'s second `a` keeps the first `a`'s
    /// position but overwrites its value -- JavaScript object-literal rules.
    #[test]
    fn duplicate_key_keeps_first_position_and_last_value() {
        // JavaScript: `{a:1, b:2, a:3}` iterates a, b -- with a === 3.
        let m: OrderedMap<u32> = serde_json::from_str(r#"{"a":1,"b":2,"a":3}"#).unwrap();
        assert_eq!(m.keys().collect::<Vec<_>>(), ["a", "b"]);
        assert_eq!(m.get("a"), Some(&3));
    }

    /// Protects: `len`/`is_empty`/`contains_key`/`get`/`values`/`iter_mut`
    /// against the values actually inserted, including that a mutation
    /// through `iter_mut` is visible to a later `get`.
    #[test]
    fn basic_accessors() {
        let mut m = OrderedMap::new();
        assert!(m.is_empty());
        m.insert("one", 1);
        m.insert("two", 2);
        assert_eq!(m.len(), 2);
        assert!(m.contains_key("two"));
        assert!(!m.contains_key("three"));
        assert_eq!(m.get("one"), Some(&1));
        assert_eq!(m.values().copied().collect::<Vec<_>>(), [1, 2]);
        for (_, v) in m.iter_mut() {
            *v *= 10;
        }
        assert_eq!(m.get("two"), Some(&20));
    }

    /// Protects: [`OrderedMap::remove`] on a missing key returns `None`
    /// without disturbing order, and a later [`OrderedMap::insert`] of the
    /// removed key is a fresh append rather than reclaiming the old slot --
    /// matching `delete`-then-reassign's effect on `Object.keys` order.
    #[test]
    fn remove_then_reinsert_moves_the_key_to_the_end() {
        let mut m = OrderedMap::new();
        m.insert("a", 1);
        m.insert("b", 2);
        m.insert("c", 3);
        assert_eq!(m.remove("a"), Some(1));
        assert_eq!(m.remove("missing"), None);
        assert_eq!(m.keys().collect::<Vec<_>>(), ["b", "c"]);
        m.insert("a", 9);
        assert_eq!(m.keys().collect::<Vec<_>>(), ["b", "c", "a"]);
        *m.get_mut("b").unwrap() += 100;
        assert_eq!(m.get("b"), Some(&102));
    }
}
