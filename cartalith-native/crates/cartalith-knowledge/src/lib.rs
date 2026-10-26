//! Retrieval knowledge base over the Markdown Vault and the world's own data
//! (`RAG_KNOWLEDGE_BASE_RESEARCH.md`, Ruling DM; milestones RK-0 to RK-3).
//!
//! The crate has no Godot type, no engine dependency, no model and no socket: it turns plain input
//! structs and a Vault into documents, indexes them with an in-repo BM25, and answers
//! `search(query)` with source-labelled hits. Generation never depends on it and it never writes.
//!
//! | Module | Owns |
//! |---|---|
//! | [`tokenize`] | the one tokenizer for index and query |
//! | [`bm25`] | the fielded in-memory BM25 index |
//! | [`model`] | the [`model::Document`], ids, the WORLD DATA / VAULT NOTE labels |
//! | [`world`] | the world-as-documents renderer and the world relation edges (RK-1) |
//! | [`vault`] | the Vault chunker, exclusion and incremental refresh (RK-2) |
//! | [`search`] | BM25 over both sources, gazetteer, link expansion, bounded results (RK-3) |
//! | [`eval`] | the evaluation set generator, scorer and substring baseline (RK-0) |

pub mod bm25;
pub mod eval;
pub mod model;
pub mod search;
pub mod tokenize;
pub mod vault;
pub mod world;
