//! Retrieval knowledge base over the Markdown Vault and the world's own data
//! (`RAG_KNOWLEDGE_BASE_RESEARCH.md`, Ruling DM; milestones RK-0 to RK-4 here, with the RK-8 to RK-11
//! measurement machinery in [`live_eval`], [`embed`], [`gate`] and [`rerank`]).
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
//! | [`embed`] | embeddings wire, vector cache, cosine, RRF, the hybrid retriever (RK-9, RK-10) |
//! | [`gate`] | the RK-9 decision gate and its stand-in controls |
//! | [`rerank`] | the RK-11 reranker and chunk-context trials |
//! | [`live_eval`] | the scorer and report for a model's runs over that set (RK-8) |

pub mod bm25;
pub mod embed;
pub mod eval;
pub mod live_eval;
pub mod gate;
pub mod model;
pub mod rerank;
pub mod search;
pub mod tokenize;
pub mod vault;
pub mod world;
