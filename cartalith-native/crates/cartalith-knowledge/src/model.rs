//! The shared document model: what a retrievable unit is, and the id scheme that names it
//! (`RAG_KNOWLEDGE_BASE_RESEARCH.md` s.5.1).
//!
//! # Ids
//!
//! Every unit has a stable string id the model can only copy:
//!
//! * world: `W:<kind>:<key>`, e.g. `W:settlement:42`, `W:faction:3`, `W:trade:42-7`;
//! * vault: `V:<path>` for a note's text before its first heading, `V:<path>#<heading path>` for a
//!   heading section, and `V:<path>#<heading path>~<n>` for the n-th (n >= 2) slice of a long
//!   section.
//!
//! [`id_matches`] is the one rule for "does a retrieved id answer a gold id": equal, or the gold id
//! is a prefix followed by `#` or `~`, so a gold `V:Settlements/Kelvhold.md` is answered by any
//! chunk of that note and a gold `V:..#The Old Quarter` by any slice of that section.
//!
//! # Authority labels
//!
//! [`Source::label`] is the citation chip text: `WORLD DATA` or `VAULT NOTE` (Ruling DM item 2).
//!
//! # What it must never do
//!
//! Mint an id from anything but the entity key or the path and heading it names, so the same input
//! always gets the same id.

use serde::{Deserialize, Serialize};

/// Which of the two knowledge sources a document came from (Ruling DM item 1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Source {
    /// Rendered from the engine's own getters: authoritative for figures.
    World,
    /// A chunk of a Markdown Vault note: canon for names and story.
    Vault,
}

impl Source {
    /// The citation chip text the panel draws (Ruling DM item 2).
    pub fn label(self) -> &'static str {
        match self {
            Source::World => "WORLD DATA",
            Source::Vault => "VAULT NOTE",
        }
    }

    /// The machine key used in tool results and the `scope` argument.
    pub fn key(self) -> &'static str {
        match self {
            Source::World => "world",
            Source::Vault => "vault",
        }
    }
}

/// One retrievable unit: a rendered world entity or one Vault chunk.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Document {
    /// The stable id (module doc).
    pub id: String,
    /// World data or Vault note.
    pub source: Source,
    /// The document kind: `settlement`, `faction`, ... for the world; `note` for the Vault.
    pub kind: String,
    /// The entity name or note title (plus aliases): the heavily weighted field.
    pub title: String,
    /// What a person can open: an entity name for the world, `path#heading` for the Vault.
    pub locator: String,
    /// The contextual header (kind and containers for the world; path, headings, type, tags,
    /// aliases and links for the Vault): indexed, lightly weighted.
    pub header: String,
    /// The text shown to the model and indexed as the body.
    pub body: String,
}

/// Whether the retrieved id `got` answers the gold id `gold` (module doc).
pub fn id_matches(gold: &str, got: &str) -> bool {
    got == gold
        || got
            .strip_prefix(gold)
            .is_some_and(|rest| rest.starts_with('#') || rest.starts_with('~'))
}

/// The id of a world document of `kind` for entity `key`.
pub fn world_id(kind: &str, key: impl std::fmt::Display) -> String {
    format!("W:{kind}:{key}")
}

/// The id of a Vault note's pre-heading text.
pub fn vault_note_id(path: &str) -> String {
    format!("V:{path}")
}

/// The id of a Vault section chunk. `slice` is 1-based; slice 1 has no suffix.
pub fn vault_chunk_id(path: &str, headings: &[String], slice: usize) -> String {
    let mut id = if headings.is_empty() { format!("V:{path}") } else { format!("V:{path}#{}", headings.join(" > ")) };
    if slice > 1 {
        id.push_str(&format!("~{slice}"));
    }
    id
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Protects the citation labels as literals (the panel and the consent text quote them).
    #[test]
    fn the_citation_labels_are_the_literals_of_ruling_dm() {
        assert_eq!(Source::World.label(), "WORLD DATA");
        assert_eq!(Source::Vault.label(), "VAULT NOTE");
        assert_eq!(Source::World.key(), "world");
        assert_eq!(Source::Vault.key(), "vault");
    }

    /// Protects id matching: a note id is answered by its chunks, a heading id by its slices, and
    /// a prefix that is merely a longer name (`Kelvhold2.md`) is NOT an answer.
    #[test]
    fn a_gold_id_matches_itself_and_its_chunks_only() {
        assert!(id_matches("V:A/Kelvhold.md", "V:A/Kelvhold.md"));
        assert!(id_matches("V:A/Kelvhold.md", "V:A/Kelvhold.md#Old Quarter"));
        assert!(id_matches("V:A/Kelvhold.md#Old Quarter", "V:A/Kelvhold.md#Old Quarter~2"));
        assert!(!id_matches("V:A/Kelvhold.md", "V:A/Kelvhold.md2"));
        assert!(!id_matches("V:A/Kelvhold.md", "V:A/Kelvhold2.md"));
        assert!(!id_matches("W:settlement:4", "W:settlement:42"));
        assert!(id_matches("W:settlement:4", "W:settlement:4"));
    }

    /// Protects the id grammar by literal.
    #[test]
    fn ids_are_built_by_the_documented_grammar() {
        assert_eq!(world_id("settlement", 42), "W:settlement:42");
        assert_eq!(vault_note_id("Settlements/Kelvhold.md"), "V:Settlements/Kelvhold.md");
        let h = vec!["Kelvhold".to_string(), "The Old Quarter".to_string()];
        assert_eq!(vault_chunk_id("S/K.md", &h, 1), "V:S/K.md#Kelvhold > The Old Quarter");
        assert_eq!(vault_chunk_id("S/K.md", &h, 3), "V:S/K.md#Kelvhold > The Old Quarter~3");
        assert_eq!(vault_chunk_id("S/K.md", &[], 1), "V:S/K.md");
    }
}
